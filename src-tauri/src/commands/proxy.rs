use std::io::{BufRead, BufReader as StdBufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex as StdMutex};
use std::thread;
use kube::config::Kubeconfig;
use tauri::State;
use tokio::time::{sleep, Duration};

use crate::KubectlProxy;

// ── kubectl binary ────────────────────────────────────────────────────────────

/// Returns the kubectl binary to invoke.
///
/// - **Windows**: always `kubectl.exe` (found via PATH).
/// - **Linux / macOS**: checks the two most common explicit paths first so the
///   proxy starts even when Tauri's stripped-down PATH doesn't include them.
fn kubectl_binary() -> String {
    if cfg!(windows) {
        return "kubectl.exe".to_string();
    }
    // Prefer explicit paths on Unix so Tauri's stripped PATH doesn't cause ENOENT.
    for candidate in ["/usr/bin/kubectl", "/usr/local/bin/kubectl"] {
        if std::path::Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "kubectl".to_string()
}

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

/// Returns all `.kube` directories to scan using `dirs::home_dir()` for
/// portability (Windows, Linux, macOS).  On WSL2 also includes the Windows-side
/// home directory.
fn kube_dirs_to_scan() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    // dirs::home_dir() is platform-aware: C:\Users\NAME on Windows, ~ on Unix.
    if let Some(home) = dirs::home_dir() {
        let d = home.join(".kube");
        if d.is_dir() {
            dirs.push(d);
        }
    }
    // WSL2: also scan the Windows-side .kube directory.
    if let Some(win_home) = windows_home_in_wsl() {
        let d = win_home.join(".kube");
        if d.is_dir() {
            dirs.push(d);
        }
    }
    dirs
}

/// Returns all regular files in `dir` whose name starts with `"config"`, sorted.
fn scan_kube_dir_for_configs(dir: &std::path::Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return vec![];
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            let name = p.file_name()?.to_str()?.to_string();
            if p.is_dir() || !name.starts_with("config") {
                return None;
            }
            Some(p)
        })
        .collect();
    paths.sort();
    paths
}

/// Fallback: scans all kubeconfig files in the applicable `.kube` directories
/// and returns the path of the first file that contains `context_name`.
/// Used when the frontend did not supply a `kubeconfig_file` (e.g. older
/// cached state or KUBECONFIG env-var contexts).
fn find_kubeconfig_for_context(context_name: &str) -> Option<PathBuf> {
    for dir in kube_dirs_to_scan() {
        eprintln!("[proxy] scanning {} for context '{context_name}'", dir.display());
        for path in scan_kube_dir_for_configs(&dir) {
            match Kubeconfig::read_from(&path) {
                Ok(cfg) => {
                    if cfg.contexts.iter().any(|c| c.name == context_name) {
                        eprintln!("[proxy] found context in {}", path.display());
                        return Some(path);
                    }
                }
                Err(e) => {
                    eprintln!("[proxy] skipping {} (parse error: {e})", path.display());
                }
            }
        }
    }
    eprintln!("[proxy] context '{context_name}' not found in any kubeconfig file");
    None
}

// ── commands ──────────────────────────────────────────────────────────────────

/// Starts `kubectl proxy --port=8001 --disable-filter=true --kubeconfig=<file> --context=<ctx>`.
///
/// Cross-platform behaviour:
/// - kubectl binary: `kubectl.exe` on Windows, explicit Unix paths on Linux/macOS.
/// - `--kubeconfig` always points to a single file (never a colon/semicolon list),
///   eliminating multi-path separator issues on all platforms.
///
/// `kubeconfig_file` (preferred): the exact file path stored in `KubeContext.kubeconfigFile`
/// by `get_kubeconfig_contexts`.  When absent, falls back to a directory scan.
///
/// Polls `http://127.0.0.1:8001/api` every 500 ms (up to 10 attempts) and
/// returns as soon as the proxy responds, rather than sleeping a fixed time.
#[tauri::command]
pub async fn start_kubectl_proxy(
    context: Option<String>,
    kubeconfig_file: Option<String>,
    state: State<'_, KubectlProxy>,
) -> Result<(), String> {
    let mut args = vec![
        "proxy".to_string(),
        "--port=8001".to_string(),
        "--disable-filter=true".to_string(),
    ];

    // Resolve kubeconfig path and context args.
    // Priority: kubeconfig_file from frontend → directory scan fallback.
    if let Some(ref ctx) = context {
        args.push(format!("--context={ctx}"));

        let resolved_path = kubeconfig_file
            .as_deref()
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
            .or_else(|| find_kubeconfig_for_context(ctx));

        if let Some(path) = resolved_path {
            // Always pass a single-file --kubeconfig; no separator ambiguity.
            args.push(format!("--kubeconfig={}", path.display()));
            eprintln!("[proxy] kubeconfig file: {}", path.display());
        } else {
            eprintln!("[proxy] warning: could not locate kubeconfig file for context '{ctx}'");
        }
    }

    // Accumulates proxy stderr lines so we can include them in error messages.
    let stderr_log: Arc<StdMutex<String>> = Arc::new(StdMutex::new(String::new()));

    let kubectl_bin = kubectl_binary();

    // ── spawn inside a block so MutexGuard is dropped before any .await ───────
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;

        // Kill any existing proxy process.
        if let Some(mut child) = guard.take() {
            eprintln!("[proxy] killing previous proxy process");
            let _ = child.kill();
        }

        eprintln!("[proxy] spawning: {kubectl_bin} {}", args.join(" "));

        let mut cmd = Command::new(&kubectl_bin);
        cmd.args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

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
                // 200 = healthy; 403 = proxy is up but auth is denied — either
                // way the proxy itself is listening and ready for kube-rs calls.
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

    // All 10 attempts failed — include whatever stderr was captured.
    let captured = stderr_log.lock().map(|g| g.clone()).unwrap_or_default();
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
