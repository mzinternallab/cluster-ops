use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use tauri::State;
use tokio::time::{sleep, Duration};

use crate::KubectlProxy;

/// Starts `kubectl proxy --port=8001 --disable-filter=true`.
/// Kills any existing proxy process first so the command is idempotent.
///
/// When `source_file` and `context` are provided, adds:
///   --kubeconfig=<source_file>  (single file — no multi-path separator issues)
///   --context=<context>
///
/// Polls http://localhost:8001/api every 500 ms (up to 10 attempts / 5 s).
/// Returns Ok(()) as soon as the proxy responds, or Err with captured stderr
/// after all attempts fail (killing the process first).
#[tauri::command]
pub async fn start_kubectl_proxy(
    context: Option<String>,
    source_file: Option<String>,
    state: State<'_, KubectlProxy>,
) -> Result<(), String> {
    let mut args = vec![
        "proxy".to_string(),
        "--port=8001".to_string(),
        "--append-server-path".to_string(),
        "--disable-filter=true".to_string(),
    ];

    if let Some(ref file) = source_file {
        args.push(format!("--kubeconfig={file}"));
    }
    if let Some(ref ctx) = context {
        args.push(format!("--context={ctx}"));
    }

    // Accumulate stderr so we can surface it on failure.
    let stderr_buf: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    // ── spawn; drop the MutexGuard before any .await ─────────────────────────
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;

        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }

        let mut child = Command::new("kubectl")
            .args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn kubectl proxy: {e}"))?;

        // Drain stderr in a background thread so the pipe never blocks.
        let buf = Arc::clone(&stderr_buf);
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().flatten() {
                    if let Ok(mut b) = buf.lock() {
                        b.push_str(&line);
                        b.push('\n');
                    }
                }
            });
        }

        *guard = Some(child);
    } // MutexGuard dropped here — safe to .await below

    // ── health-check loop ─────────────────────────────────────────────────────
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(400))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    for attempt in 1..=10u32 {
        sleep(Duration::from_millis(500)).await;

        match client.get("http://localhost:8001/api").send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                // 200 = healthy; 403 = proxy up but auth denied — either way it's listening.
                if resp.status().is_success() || status == 403 {
                    return Ok(());
                }
            }
            Err(_) => {} // not up yet — keep polling
        }

        // If the process has already exited, stop polling early.
        let exited = {
            let mut guard = state.0.lock().map_err(|e| e.to_string())?;
            guard
                .as_mut()
                .and_then(|c| c.try_wait().ok())
                .map(|s| s.is_some())
                .unwrap_or(false)
        };
        if exited {
            break;
        }

        eprintln!("[proxy] attempt {attempt}/10: not ready yet");
    }

    // All attempts failed — kill the process and return stderr as the error.
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }
    }

    let captured = stderr_buf.lock().map(|b| b.clone()).unwrap_or_default();
    let detail = if captured.trim().is_empty() {
        "no output from kubectl proxy".to_string()
    } else {
        captured
    };

    Err(format!(
        "kubectl proxy did not become ready after 10 attempts.\nProxy output:\n{detail}"
    ))
}

/// Kills the running kubectl proxy process.
/// The frontend calls this on window unload; lib.rs also calls it on RunEvent::Exit.
#[tauri::command]
pub async fn stop_kubectl_proxy(state: State<'_, KubectlProxy>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = guard.take() {
        child.kill().map_err(|e| e.to_string())?;
    }
    Ok(())
}
