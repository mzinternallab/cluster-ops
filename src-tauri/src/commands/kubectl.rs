// Spawn kubectl subprocess and stream output — Phase 1 Step 9

/// Runs kubectl describe on a pod and returns the output.
#[tauri::command]
pub async fn describe_pod(_name: String, _namespace: String) -> Result<String, String> {
    // TODO: implement in Phase 1 Step 9
    Ok(String::new())
}

/// Runs an arbitrary kubectl command and streams output via Tauri events.
#[tauri::command]
pub async fn run_kubectl(_command: String) -> Result<(), String> {
    // TODO: implement in Phase 1 Step 13
    Ok(())
}
