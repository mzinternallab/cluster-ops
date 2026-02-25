use std::process::Stdio;

use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

// ── get_pod_logs ──────────────────────────────────────────────────────────────

/// Streams pod logs line-by-line via Tauri events.
///
/// Events emitted:
/// - `pod-log-line`  — payload: `String`  — one line of output
/// - `pod-log-error` — payload: `String`  — kubectl stderr (on non-zero exit)
/// - `pod-log-done`  — payload: `null`    — stream finished
#[tauri::command]
pub async fn get_pod_logs(
    app: AppHandle,
    name: String,
    namespace: String,
    source_file: String,
    context_name: String,
    tail: Option<u32>,
    follow: bool,
) -> Result<(), String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    let mut args = vec![
        "logs".to_string(),
        name,
        "-n".to_string(),
        namespace,
        format!("--kubeconfig={source_file}"),
        format!("--context={context_name}"),
    ];

    if let Some(n) = tail {
        args.push(format!("--tail={n}"));
    }
    if follow {
        args.push("-f".to_string());
    }

    let mut child = Command::new(&kubectl)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("kubectl not found: {e}"))?;

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let mut lines = BufReader::new(stdout).lines();

    while let Some(line) = lines.next_line().await.map_err(|e| e.to_string())? {
        app.emit("pod-log-line", line).map_err(|e| e.to_string())?;
    }

    drop(lines);

    let output = child.wait_with_output().await.map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let err = err.trim();
        if !err.is_empty() {
            app.emit("pod-log-error", err).map_err(|e| e.to_string())?;
        }
    }

    app.emit("pod-log-done", ()).map_err(|e| e.to_string())?;
    Ok(())
}
