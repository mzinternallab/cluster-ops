use tauri::{AppHandle, Emitter};
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

/// Runs an arbitrary kubectl command, appending --kubeconfig and --context.
/// Streams output line-by-line via `command-output-line` / `command-output-error`
/// events, then emits `command-output-done`.
#[tauri::command]
pub async fn run_kubectl(
    app: AppHandle,
    command: String,
    source_file: String,
    context_name: String,
) -> Result<(), String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    // Strip leading "kubectl" / "kubectl " so the user can type either form
    let args_str = command
        .trim()
        .trim_start_matches("kubectl")
        .trim_start();

    let mut args = shell_words::split(args_str).map_err(|e| e.to_string())?;
    args.push(format!("--kubeconfig={source_file}"));
    args.push(format!("--context={context_name}"));

    eprintln!("[kubectl] {} {}", kubectl, args.join(" "));

    let output = Command::new(&kubectl)
        .args(&args)
        .output()
        .await
        .map_err(|e| format!("kubectl not found: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    for line in stdout.lines() {
        app.emit("command-output-line", line).map_err(|e| e.to_string())?;
    }
    if !stderr.is_empty() {
        for line in stderr.lines() {
            app.emit("command-output-error", line).map_err(|e| e.to_string())?;
        }
    }
    app.emit("command-output-done", ()).map_err(|e| e.to_string())?;
    Ok(())
}
