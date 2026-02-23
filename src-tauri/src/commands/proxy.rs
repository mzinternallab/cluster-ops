use std::path::PathBuf;
use std::process::{Command, Stdio};
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
/// - Sleeps 1500 ms before returning so the caller can safely issue API
///   requests as soon as this promise resolves.
#[tauri::command]
pub async fn start_kubectl_proxy(
    context: Option<String>,
    state: State<'_, KubectlProxy>,
) -> Result<(), String> {
    // Spawn inside a block so the MutexGuard is dropped before we .await.
    // std::sync::MutexGuard is !Send, so it must not be held across any await point.
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;

        // Kill any existing proxy process.
        if let Some(mut child) = guard.take() {
            let _ = child.kill();
        }

        let mut args = vec![
            "proxy".to_string(),
            "--port=8001".to_string(),
            "--disable-filter=true".to_string(),
        ];
        if let Some(ref ctx) = context {
            eprintln!("[proxy] starting with --context={ctx}");
            args.push(format!("--context={ctx}"));
        } else {
            eprintln!("[proxy] starting without explicit context (using current-context from kubeconfig)");
        }

        let mut cmd = Command::new("kubectl");
        cmd.args(&args)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // Override KUBECONFIG so kubectl can find the Windows-side config files.
        if let Some(kubeconfig) = build_kubeconfig_env() {
            cmd.env("KUBECONFIG", kubeconfig);
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn kubectl proxy: {e}"))?;

        *guard = Some(child);
    } // guard is dropped here — safe to .await below

    // Give the proxy 1500 ms to start listening on :8001 before returning,
    // so the frontend can fire API requests immediately after this resolves.
    sleep(Duration::from_millis(1500)).await;

    eprintln!("[proxy] ready on :8001");
    Ok(())
}

/// Kills the running kubectl proxy process.
/// Called on window unload and on RunEvent::Exit.
#[tauri::command]
pub async fn stop_kubectl_proxy(state: State<'_, KubectlProxy>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = guard.take() {
        child.kill().map_err(|e| e.to_string())?;
    }
    Ok(())
}
