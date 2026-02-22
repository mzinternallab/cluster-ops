use tokio::process::Command;

// ── describe_pod ──────────────────────────────────────────────────────────────

/// Runs `kubectl describe pod <name> -n <namespace>` and returns the full output.
#[tauri::command]
pub async fn describe_pod(name: String, namespace: String) -> Result<String, String> {
    let output = Command::new("kubectl")
        .args(["describe", "pod", &name, "-n", &namespace])
        .output()
        .await
        .map_err(|e| format!("kubectl not found: {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("kubectl: {}", err.trim()))
    }
}

// ── run_kubectl ───────────────────────────────────────────────────────────────

/// Runs an arbitrary kubectl command and streams output via Tauri events.
/// Implemented in Phase 1 Step 13.
#[tauri::command]
pub async fn run_kubectl(_command: String) -> Result<(), String> {
    // TODO: implement in Phase 1 Step 13
    Ok(())
}
