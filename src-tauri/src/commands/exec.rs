use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tauri::{AppHandle, Emitter, State};

// ── PTY session state ─────────────────────────────────────────────────────────

/// All handles needed to drive a live PTY session.
pub struct PtySessionInner {
    /// Write end — receives keystrokes from the frontend.
    pub writer: Box<dyn Write + Send>,
    /// Master PTY handle — used for resize.  Kept alive so the slave stays open.
    pub master: Box<dyn portable_pty::MasterPty>,
    /// Child process handle — used for kill.
    pub child: Box<dyn portable_pty::Child>,
}

// SAFETY: portable-pty's concrete Master and Child implementations are all
// thread-safe.  We only ever access them while holding the Mutex lock.
unsafe impl Send for PtySessionInner {}
unsafe impl Sync for PtySessionInner {}

/// Tauri managed state — holds the current PTY session (if any).
pub struct PtySession(pub Arc<Mutex<Option<PtySessionInner>>>);

// ── commands ──────────────────────────────────────────────────────────────────

/// Opens a PTY and spawns `kubectl exec -it` through it.
///
/// - PTY size defaults to 80×24; the frontend calls `resize_pty` immediately
///   after mount to sync the real terminal dimensions.
/// - Streams raw PTY output (ANSI escape sequences included) to the frontend
///   via `pty-output` events.  The frontend passes bytes directly to xterm.js.
/// - Emits `pty-done` when the process exits or the read loop ends.
#[tauri::command]
pub async fn start_pty_exec(
    app: AppHandle,
    name: String,
    namespace: String,
    source_file: String,
    context_name: String,
    cols: Option<u16>,
    rows: Option<u16>,
    state: State<'_, PtySession>,
) -> Result<(), String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    eprintln!(
        "[pty] {kubectl} exec -it {name} -n {namespace} \
         --kubeconfig={source_file} --context={context_name} -- /bin/sh"
    );

    let pty_size = PtySize {
        rows: rows.unwrap_or(24),
        cols: cols.unwrap_or(80),
        pixel_width: 0,
        pixel_height: 0,
    };

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(pty_size)
        .map_err(|e| format!("failed to open PTY: {e}"))?;

    let mut cmd = CommandBuilder::new(&kubectl);
    cmd.args([
        "exec",
        "-it",
        &name,
        "-n",
        &namespace,
        &format!("--kubeconfig={source_file}"),
        &format!("--context={context_name}"),
        "--",
        "/bin/sh",
    ]);

    let child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| format!("failed to spawn kubectl exec: {e}"))?;

    // Clone reader before taking writer (both borrow master immutably).
    let reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| format!("failed to clone PTY reader: {e}"))?;

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| format!("failed to take PTY writer: {e}"))?;

    // Replace any previous session.
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        if let Some(mut prev) = guard.take() {
            let _ = prev.child.kill();
        }
        *guard = Some(PtySessionInner {
            writer,
            master: pair.master,
            child,
        });
    }

    // Background thread: read raw PTY bytes and emit them as events.
    let app_clone = app.clone();
    std::thread::spawn(move || {
        eprintln!("[pty-read] reader thread started");
        let mut reader = reader;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    eprintln!("[pty-read] EOF (read 0 bytes) — exiting");
                    break;
                }
                Ok(n) => {
                    eprintln!("[pty-read] got {n} bytes");
                    // Send raw bytes as a lossy UTF-8 string so xterm.js receives
                    // ANSI escape sequences unchanged.
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    if app_clone.emit("pty-output", data).is_err() {
                        eprintln!("[pty-read] emit error — exiting");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("[pty-read] read error: {e} — exiting");
                    break;
                }
            }
        }
        eprintln!("[pty-read] thread done, emitting pty-done");
        let _ = app_clone.emit("pty-done", ());
    });

    Ok(())
}

/// Sends raw bytes (keystrokes, control sequences) to the PTY writer.
/// Called on every xterm.js `onData` event — must be fast / non-blocking.
#[tauri::command]
pub async fn send_exec_input(
    data: String,
    state: State<'_, PtySession>,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut session) = *guard {
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|e| format!("PTY write error: {e}"))?;
        session
            .writer
            .flush()
            .map_err(|e| format!("PTY flush error: {e}"))?;
    }
    Ok(())
}

/// Resizes the PTY to match the current xterm.js terminal dimensions.
/// Called after fit() reports new cols/rows.
#[tauri::command]
pub async fn resize_pty(
    cols: u16,
    rows: u16,
    state: State<'_, PtySession>,
) -> Result<(), String> {
    let guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(ref session) = *guard {
        session
            .master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| format!("PTY resize error: {e}"))?;
    }
    Ok(())
}

/// Kills the running PTY session.
/// Called on component unmount and when switching away from exec mode.
#[tauri::command]
pub async fn stop_pty_exec(state: State<'_, PtySession>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut session) = guard.take() {
        let _ = session.child.kill();
    }
    Ok(())
}
