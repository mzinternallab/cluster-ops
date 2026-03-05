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

// ── get_pod_describe_for_security ────────────────────────────────────────────

/// Alias for describe_pod used by the security scan feature.
/// Returns the full kubectl describe pod output as a String.
#[tauri::command]
pub async fn get_pod_describe_for_security(
    name: String,
    namespace: String,
    source_file: String,
    context_name: String,
) -> Result<String, String> {
    describe_pod(name, namespace, source_file, context_name).await
}

// ── get_network_scan_data ─────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_network_scan_data(
    namespace: String,
    source_file: String,
    context_name: String,
) -> Result<String, String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    let kubeconfig = format!("--kubeconfig={source_file}");
    let context    = format!("--context={context_name}");

    let commands: Vec<Vec<&str>> = vec![
        vec!["get", "networkpolicy", "-n", &namespace, "-o", "yaml"],
        vec!["get", "pods",          "-n", &namespace, "-o", "wide"],
        vec!["get", "services",      "-n", &namespace, "-o", "yaml"],
        vec!["get", "ingress",       "-n", &namespace, "-o", "yaml"],
    ];

    let mut combined = String::new();
    for args in &commands {
        let mut full_args = args.clone();
        full_args.push(&kubeconfig);
        full_args.push(&context);

        if let Ok(output) = tokio::process::Command::new(&kubectl)
            .args(&full_args)
            .output()
            .await
        {
            combined.push_str(&format!("=== {} ===\n", args.join(" ")));
            combined.push_str(&String::from_utf8_lossy(&output.stdout));
            combined.push('\n');
        }
    }
    Ok(combined)
}

// ── get_rbac_scan_data ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_rbac_scan_data(
    namespace: String,
    source_file: String,
    context_name: String,
) -> Result<String, String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    let kubeconfig = format!("--kubeconfig={source_file}");
    let context    = format!("--context={context_name}");

    let commands: Vec<Vec<&str>> = vec![
        vec!["get", "rolebindings",        "-n", &namespace, "-o", "yaml"],
        vec!["get", "roles",               "-n", &namespace, "-o", "yaml"],
        vec!["get", "serviceaccounts",     "-n", &namespace, "-o", "yaml"],
        vec!["get", "clusterrolebindings",               "-o", "yaml"],
    ];

    let mut combined = String::new();
    for args in &commands {
        let mut full_args = args.clone();
        full_args.push(&kubeconfig);
        full_args.push(&context);

        if let Ok(output) = tokio::process::Command::new(&kubectl)
            .args(&full_args)
            .output()
            .await
        {
            combined.push_str(&format!("=== {} ===\n", args.join(" ")));
            combined.push_str(&String::from_utf8_lossy(&output.stdout));
            combined.push('\n');
        }
    }
    Ok(combined)
}

// ── get_namespace_scan_data ───────────────────────────────────────────────────

#[tauri::command]
pub async fn get_namespace_scan_data(
    namespace: String,
    source_file: String,
    context_name: String,
) -> Result<String, String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    let kubeconfig = format!("--kubeconfig={source_file}");
    let context    = format!("--context={context_name}");

    let commands: Vec<Vec<&str>> = vec![
        vec!["get", "resourcequota",      "-n", &namespace, "-o", "yaml"],
        vec!["get", "limitrange",         "-n", &namespace, "-o", "yaml"],
        vec!["get", "namespace",          &namespace,       "-o", "yaml"],
        vec!["get", "configmap",          "-n", &namespace],
        vec!["get", "podsecuritypolicy",                    "-o", "yaml"],
    ];

    let mut combined = String::new();
    for args in &commands {
        let mut full_args = args.clone();
        full_args.push(&kubeconfig);
        full_args.push(&context);

        if let Ok(output) = tokio::process::Command::new(&kubectl)
            .args(&full_args)
            .output()
            .await
        {
            combined.push_str(&format!("=== {} ===\n", args.join(" ")));
            combined.push_str(&String::from_utf8_lossy(&output.stdout));
            combined.push('\n');
        }
    }
    Ok(combined)
}

// ── get_node_scan_data ────────────────────────────────────────────────────────

#[tauri::command]
pub async fn get_node_scan_data(
    source_file: String,
    context_name: String,
) -> Result<String, String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    let kubeconfig = format!("--kubeconfig={source_file}");
    let context    = format!("--context={context_name}");

    let commands: Vec<Vec<&str>> = vec![
        vec!["get",      "nodes",             "-o", "wide"],
        vec!["describe", "nodes"],
        vec!["get",      "pods", "--all-namespaces", "-o", "wide"],
        vec!["version",  "--short"],
    ];

    let mut combined = String::new();
    for args in &commands {
        let mut full_args = args.clone();
        full_args.push(&kubeconfig);
        full_args.push(&context);

        if let Ok(output) = tokio::process::Command::new(&kubectl)
            .args(&full_args)
            .output()
            .await
        {
            combined.push_str(&format!("=== {} ===\n", args.join(" ")));
            combined.push_str(&String::from_utf8_lossy(&output.stdout));
            combined.push('\n');
        }
    }
    Ok(combined)
}

// ── run_kubectl ───────────────────────────────────────────────────────────────

/// Runs an arbitrary kubectl command through the system shell so that pipes,
/// grep, and other shell features work correctly.
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

    // Build the full command string with kubeconfig and context
    // injected after "kubectl" but before any pipes or redirects
    let full_command = inject_kubectl_flags(&command, &kubectl, &source_file, &context_name);

    // Run through system shell so pipes, grep, etc. work.
    // PowerShell is used on Windows instead of cmd to avoid buffering
    // issues with piped commands. Wrapped in a 30-second timeout to
    // prevent infinite hangs.
    let output = tokio::time::timeout(
        std::time::Duration::from_secs(30),
        async {
            if cfg!(windows) {
                tokio::process::Command::new("powershell")
                    .args(["-NoProfile", "-NonInteractive", "-Command", &full_command])
                    .output()
                    .await
            } else {
                tokio::process::Command::new("sh")
                    .args(["-c", &full_command])
                    .output()
                    .await
            }
        },
    )
    .await
    .map_err(|_| "Command timed out after 30 seconds".to_string())?
    .map_err(|e| format!("shell error: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    for line in stdout.lines() {
        app.emit("command-output-line", line)
            .map_err(|e| e.to_string())?;
    }
    if !stderr.is_empty() {
        for line in stderr.lines() {
            app.emit("command-output-error", line)
                .map_err(|e| e.to_string())?;
        }
    }
    app.emit("command-output-done", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

// Inject --kubeconfig and --context flags into the kubectl
// portion of the command, before any pipe or redirect.
// Handles: "kubectl get pods | grep foo"
// Becomes: "/path/to/kubectl get pods --kubeconfig=X --context=Y | grep foo"
fn inject_kubectl_flags(
    command: &str,
    kubectl: &str,
    source_file: &str,
    context_name: &str,
) -> String {
    let flags = format!(
        "--kubeconfig={} --context={}",
        source_file, context_name
    );

    // Strip leading "kubectl" from command
    let stripped = command
        .trim()
        .trim_start_matches("kubectl ")
        .trim_start_matches("kubectl");

    // Find the first pipe or redirect in the stripped command
    let pipe_pos = stripped.find(" | ")
        .or_else(|| stripped.find(" > "))
        .or_else(|| stripped.find(" >> "));

    if let Some(p) = pipe_pos {
        let (before_pipe, after_pipe) = stripped.split_at(p);
        format!("{} {} {} {}", kubectl, before_pipe, flags, after_pipe)
    } else {
        format!("{} {} {}", kubectl, stripped, flags)
    }
}
