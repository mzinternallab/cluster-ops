use std::io::Read;
use tauri::{AppHandle, Emitter, State};
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

/// Runs an arbitrary kubectl command through a PTY-backed shell so that pipes,
/// grep, awk, and other shell features work correctly.
/// Streams output line-by-line via `command-output-line` events, then emits
/// `command-output-done`.
#[tauri::command]
pub async fn run_kubectl(
    app: AppHandle,
    command: String,
    source_file: String,
    context_name: String,
    state: State<'_, crate::PtyState>,
) -> Result<(), String> {
    // Clear any previous PTY writer so stale exec sessions don't linger.
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;
        *guard = None;
    }

    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    // Build full command with kubeconfig flags injected before any pipes.
    let full_cmd = inject_kubectl_flags(&command, &kubectl, &source_file, &context_name);

    // portable-pty is synchronous — open the PTY and spawn inside spawn_blocking.
    let (reader, child, slave) = tokio::task::spawn_blocking(move || {
        let pty_system = portable_pty::native_pty_system();
        let pty_pair = pty_system
            .openpty(portable_pty::PtySize {
                rows: 24,
                cols: 220,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|e| e.to_string())?;

        // Run through the system shell so pipes/redirects are handled natively.
        // KUBECONFIG env var is also set so kubectl finds the right cluster.
        let mut cmd = if cfg!(windows) {
            let mut c = portable_pty::CommandBuilder::new("cmd");
            c.args(["/C", &full_cmd]);
            c
        } else {
            let mut c = portable_pty::CommandBuilder::new("sh");
            c.args(["-c", &full_cmd]);
            c
        };
        cmd.env("KUBECONFIG", &source_file);

        let child = pty_pair
            .slave
            .spawn_command(cmd)
            .map_err(|e| format!("spawn failed: {e}"))?;

        let reader = pty_pair
            .master
            .try_clone_reader()
            .map_err(|e| e.to_string())?;

        // Move slave out so the reader closure can keep it alive.
        let slave = pty_pair.slave;

        Ok::<_, String>((reader, child, slave))
    })
    .await
    .map_err(|e| e.to_string())??;

    // Stream PTY output to the frontend line-by-line.
    // child and slave are moved in to keep the process and PTY fd alive.
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        let _child = child;
        let _slave = slave;
        let mut reader = reader;
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let data = String::from_utf8_lossy(&buf[..n]).to_string();
                    for line in data.lines() {
                        let _ = app_clone.emit("command-output-line", line.to_string());
                    }
                }
                Err(_) => break,
            }
        }
        let _ = app_clone.emit("command-output-done", ());
    });

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
