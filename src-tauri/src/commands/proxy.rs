use std::process::{Command, Stdio};
use tauri::State;

use crate::KubectlProxy;

/// Starts `kubectl proxy --port=8001 --disable-filter=true`.
/// Kills any existing proxy process first so the command is idempotent.
///
/// When `source_file` and `context` are provided, adds:
///   --kubeconfig=<source_file>  (single file — no multi-path separator issues)
///   --context=<context>
#[tauri::command]
pub async fn start_kubectl_proxy(
    context: Option<String>,
    source_file: Option<String>,
    state: State<'_, KubectlProxy>,
) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;

    // Kill any existing proxy process before (re-)starting.
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
    }

    let mut args = vec![
        "proxy".to_string(),
        "--port=8001".to_string(),
        "--disable-filter=true".to_string(),
    ];

    if let Some(ref file) = source_file {
        args.push(format!("--kubeconfig={file}"));
    }
    if let Some(ref ctx) = context {
        args.push(format!("--context={ctx}"));
    }

    let child = Command::new("kubectl")
        .args(&args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("failed to spawn kubectl proxy: {e}"))?;

    *guard = Some(child);
    Ok(())
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
