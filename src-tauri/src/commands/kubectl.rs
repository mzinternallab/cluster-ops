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

    // Strip leading "kubectl" if present
    let cmd_body = command.trim()
        .trim_start_matches("kubectl")
        .trim();

    // Split on first pipe if present
    let (kubectl_part, pipe_part) = if let Some(idx) = cmd_body.find(" | ") {
        (&cmd_body[..idx], Some(cmd_body[idx + 3..].trim()))
    } else {
        (cmd_body, None)
    };

    // Parse kubectl args
    let mut args: Vec<String> = kubectl_part
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    args.push(format!("--kubeconfig={source_file}"));
    args.push(format!("--context={context_name}"));

    // Run kubectl directly
    let output = tokio::time::timeout(
        tokio::time::Duration::from_secs(30),
        tokio::process::Command::new(&kubectl)
            .args(&args)
            .output()
    ).await
    .map_err(|_| "Command timed out after 30 seconds".to_string())?
    .map_err(|e| format!("kubectl error: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Apply pipe filter in Rust if present
    let final_output = if let Some(pipe_cmd) = pipe_part {
        apply_pipe(stdout.trim(), pipe_cmd)
    } else {
        stdout
    };

    for line in final_output.lines() {
        app.emit("command-output-line", line.to_string())
            .map_err(|e| e.to_string())?;
    }
    if !stderr.is_empty() {
        for line in stderr.lines() {
            app.emit("command-output-error", line.to_string())
                .map_err(|e| e.to_string())?;
        }
    }
    app.emit("command-output-done", ())
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn apply_pipe(input: &str, pipe_cmd: &str) -> String {
    let p = pipe_cmd.trim();

    if let Some(pat) = p.strip_prefix("grep -v ") {
        let pat = pat.trim().trim_matches('\'').trim_matches('"');
        return input.lines()
            .filter(|l| !l.contains(pat))
            .collect::<Vec<_>>().join("\n");
    }
    if let Some(pat) = p.strip_prefix("grep -i ") {
        let pat = pat.trim().trim_matches('\'').trim_matches('"').to_lowercase();
        return input.lines()
            .filter(|l| l.to_lowercase().contains(&pat))
            .collect::<Vec<_>>().join("\n");
    }
    if let Some(pat) = p.strip_prefix("grep ") {
        let pat = pat.trim().trim_matches('\'').trim_matches('"');
        return input.lines()
            .filter(|l| l.contains(pat))
            .collect::<Vec<_>>().join("\n");
    }
    if let Some(n) = p.strip_prefix("tail -") {
        if let Ok(n) = n.trim().parse::<usize>() {
            let lines: Vec<&str> = input.lines().collect();
            return lines[lines.len().saturating_sub(n)..].join("\n");
        }
    }
    if let Some(n) = p.strip_prefix("head -") {
        if let Ok(n) = n.trim().parse::<usize>() {
            return input.lines().take(n)
                .collect::<Vec<_>>().join("\n");
        }
    }
    if p == "wc -l" {
        return input.lines().count().to_string();
    }

    // Unsupported pipe — show output with note
    format!("{}\n\n[ClusterOps: '{}' not supported. Use: grep, grep -v, grep -i, tail -n, head -n, wc -l]",
        input, pipe_cmd)
}
