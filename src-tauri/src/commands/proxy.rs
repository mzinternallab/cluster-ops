use std::io::{BufRead, BufReader as StdBufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use tauri::State;
use tokio::time::{sleep, Duration};

use crate::KubectlProxy;

// ── WSL / Windows helpers ─────────────────────────────────────────────────────

/// In WSL2 the `USERPROFILE` env var is inherited from Windows (e.g. `C:\Users\7va`).
/// Converts it to the equivalent `/mnt/c/Users/7va` path so we can scan the
/// Windows-side `.kube` directory.  Returns `None` if not running under WSL2.
fn windows_home_in_wsl() -> Option<PathBuf> {
    let profile = std::env::var("USERPROFILE").ok()?;
    let profile = profile.trim();
    let mut chars = profile.chars();
    let drive = chars.next()?.to_ascii_lowercase();
    if chars.next()? != ':' {
        return None;
    }
    let rest = profile[2..].replace('\\', "/");
    Some(PathBuf::from(format!("/mnt/{drive}{rest}")))
}

/// Builds a colon-separated KUBECONFIG value from every regular, non-hidden file
/// found in `~/.kube` (WSL home) and `%USERPROFILE%\.kube` (Windows, if in WSL2).
/// This is passed as the KUBECONFIG env var when spawning kubectl proxy so that
/// kubectl can find kubeconfig files on the Windows filesystem.
fn build_kubeconfig_env() -> Option<String> {
    let mut kube_dirs: Vec<PathBuf> = Vec::new();

    if let Some(home) = dirs::home_dir() {
        kube_dirs.push(home.join(".kube"));
    }
    if let Some(win_home) = windows_home_in_wsl() {
        kube_dirs.push(win_home.join(".kube"));
    }

    let mut files: Vec<String> = Vec::new();
    for dir in &kube_dirs {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        let mut dir_files: Vec<String> = entries
            .flatten()
            .filter_map(|e| {
                let p = e.path();
                if p.is_dir() {
                    return None;
                }
                let name = p.file_name()?.to_str()?.to_string();
                if name.starts_with('.') {
                    return None;
                }
                Some(p.to_string_lossy().into_owned())
            })
            .collect();
        dir_files.sort();
        files.extend(dir_files);
    }

    if files.is_empty() {
        None
    } else {
        eprintln!("[proxy] KUBECONFIG = {}", files.join(":"));
        Some(files.join(":"))
    }
}

// ── commands ──────────────────────────────────────────────────────────────────

/// Starts `kubectl proxy --port=8001 --disable-filter=true [--context=<ctx>]`.
///
/// - Kills any running proxy first (idempotent).
/// - Sets `KUBECONFIG` to include all kubeconfig files found in `~/.kube`
///   and the Windows-side `%USERPROFILE%\.kube` (WSL2 support).
/// - Passes `--context` when the caller supplies one (cluster-switch path).
///   At startup, omit `context` and kubectl uses the `current-context` from
///   the merged kubeconfig.
/// - Polls http://127.0.0.1:8001/api every 500 ms (up to 10 attempts) instead
///   of sleeping a fixed duration, so the frontend unblocks as soon as the
///   proxy is actually ready.
#[tauri::command]
pub async fn start_kubectl_proxy(
    context: Option<String>,
    state: State<'_, KubectlProxy>,
) -> Result<(), String> {
    let mut args = vec![
        "proxy".to_string(),
        "--port=8001".to_string(),
        "--disable-filter=true".to_string(),
    ];
    if let Some(ref ctx) = context {
        args.push(format!("--context={ctx}"));
    }

    // Accumulates proxy stderr lines so we can include them in error messages.
    let stderr_log: Arc<StdMutex<String>> = Arc::new(StdMutex::new(String::new()));

    // ── spawn inside a block so MutexGuard is dropped before any .await ───────
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;

        // Kill any existing proxy process.
        if let Some(mut child) = guard.take() {
            eprintln!("[proxy] killing previous proxy process");
            let _ = child.kill();
        }

        // Resolve kubectl binary — use the WSL path when available, else PATH.
        let kubectl_bin = if cfg!(target_os = "linux") {
            // In WSL2 the binary lives at /usr/bin/kubectl (installed via apt / snap).
            // Prefer the explicit path so Tauri's stripped-down PATH isn't an issue.
            if std::path::Path::new("/usr/bin/kubectl").exists() {
                "/usr/bin/kubectl".to_string()
            } else if std::path::Path::new("/usr/local/bin/kubectl").exists() {
                "/usr/local/bin/kubectl".to_string()
            } else {
                "kubectl".to_string()
            }
        } else {
            "kubectl".to_string()
        };

        eprintln!(
            "[proxy] spawning: {} {}",
            kubectl_bin,
            args.join(" ")
        );

        let mut cmd = Command::new(&kubectl_bin);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Override KUBECONFIG so kubectl can find the Windows-side config files.
        if let Some(kubeconfig) = build_kubeconfig_env() {
            cmd.env("KUBECONFIG", kubeconfig);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn kubectl proxy ({kubectl_bin}): {e}"))?;

        eprintln!("[proxy] process spawned (pid {:?})", child.id());

        // ── drain stdout in a background thread ──────────────────────────────
        if let Some(stdout) = child.stdout.take() {
            thread::spawn(move || {
                for line in StdBufReader::new(stdout).lines().flatten() {
                    eprintln!("[proxy stdout] {line}");
                }
            });
        }

        // ── drain stderr in a background thread, accumulate for errors ───────
        let log_clone = Arc::clone(&stderr_log);
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                for line in StdBufReader::new(stderr).lines().flatten() {
                    eprintln!("[proxy stderr] {line}");
                    if let Ok(mut buf) = log_clone.lock() {
                        buf.push_str(&line);
                        buf.push('\n');
                    }
                }
            });
        }

        *guard = Some(child);
    } // guard is dropped here — safe to .await below

    // ── health-check loop: poll /api every 500 ms, up to 10 attempts ─────────
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(400))
        .build()
        .map_err(|e| format!("failed to build HTTP client: {e}"))?;

    for attempt in 1..=10u32 {
        sleep(Duration::from_millis(500)).await;

        match client.get("http://127.0.0.1:8001/api").send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                eprintln!("[proxy] attempt {attempt}/10: HTTP {status}");
                // 200 means healthy; 403 means proxy is up but auth is denied —
                // either way the proxy itself is listening.
                if resp.status().is_success() || status == 403 {
                    eprintln!("[proxy] ready on :8001 (after {attempt} attempt(s))");
                    return Ok(());
                }
            }
            Err(e) => {
                eprintln!("[proxy] attempt {attempt}/10: {e}");
            }
        }
    }

    // All 10 attempts failed — collect whatever stderr we captured.
    let captured = stderr_log
        .lock()
        .map(|g| g.clone())
        .unwrap_or_default();

    let detail = if captured.trim().is_empty() {
        "no output captured from kubectl proxy".to_string()
    } else {
        captured
    };

    Err(format!(
        "kubectl proxy did not become ready after 10 attempts (5 s).\nProxy output:\n{detail}"
    ))
}

/// Kills the running kubectl proxy process.
/// Called on window unload and on RunEvent::Exit.
#[tauri::command]
pub async fn stop_kubectl_proxy(state: State<'_, KubectlProxy>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = guard.take() {
        eprintln!("[proxy] stopping proxy process");
        child.kill().map_err(|e| e.to_string())?;
    }
    Ok(())
}
