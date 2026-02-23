use tokio::process::Command;

// ── describe_pod ──────────────────────────────────────────────────────────────

/// Runs `kubectl describe pod <name> -n <namespace>` against the specific
/// kubeconfig file and context for the active cluster.
#[tauri::command]
pub async fn describe_pod(
    name: String,
    namespace: String,
    source_file: String,
    context_name: String,
) -> Result<String, String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    eprintln!(
        "[describe] {} describe pod {name} -n {namespace} --kubeconfig={source_file} --context={context_name}",
        kubectl
    );

    let output = Command::new(&kubectl)
        .args([
            "describe", "pod", &name,
            "-n", &namespace,
            &format!("--kubeconfig={source_file}"),
            &format!("--context={context_name}"),
        ])
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

#[tauri::command]
pub async fn run_kubectl(_command: String) -> Result<(), String> {
    // TODO: implement in Phase 1 Step 13
    Ok(())
}
