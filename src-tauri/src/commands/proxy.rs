use std::process::{Command, Stdio};
use tauri::State;

use crate::KubectlProxy;

/// Starts `kubectl proxy --port=8001 --disable-filter=true`.
/// Kills any existing proxy process first so the command is idempotent.
/// The frontend calls this on app startup before making any API requests.
#[tauri::command]
pub async fn start_kubectl_proxy(state: State<'_, KubectlProxy>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;

    // Kill any existing proxy process before (re-)starting.
    if let Some(mut child) = guard.take() {
        let _ = child.kill();
    }

    let child = Command::new("kubectl")
        .args(["proxy", "--port=8001", "--disable-filter=true"])
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
