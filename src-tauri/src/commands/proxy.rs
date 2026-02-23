use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use tauri::State;
use tokio::time::{sleep, Duration};

use crate::KubectlProxy;

#[tauri::command]
pub async fn start_kubectl_proxy(
    context_name: Option<String>,
    source_file: Option<String>,
    state: State<'_, KubectlProxy>,
) -> Result<(), String> {
    let mut args = vec![
        "proxy".to_string(),
        "--port=8001".to_string(),
        "--append-server-path".to_string(),
        "--disable-filter=true".to_string(),
    ];

    if let Some(ref file) = source_file {
        args.push(format!("--kubeconfig={file}"));
    }
    if let Some(ref ctx) = context_name {
        args.push(format!("--context={ctx}"));
    }

    // ── resolve kubectl binary ────────────────────────────────────────────────
    let kubectl_path = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            if cfg!(windows) {
                let candidates = vec![
                    format!(
                        "{}\\development_tools\\kubectl.exe",
                        std::env::var("USERPROFILE").unwrap_or_default()
                    ),
                    "C:\\Program Files\\Docker\\Docker\\resources\\bin\\kubectl.exe".to_string(),
                    "kubectl.exe".to_string(),
                ];
                candidates
                    .into_iter()
                    .find(|p| std::path::Path::new(p).exists())
                    .unwrap_or_else(|| "kubectl.exe".to_string())
            } else {
                "kubectl".to_string()
            }
        });

    if !std::path::Path::new(&kubectl_path).exists() && which::which("kubectl").is_err() {
        return Err(
            "kubectl not found. Please install kubectl and ensure it is in your PATH."
                .to_string(),
        );
    }

    eprintln!("[proxy] kubectl path: {kubectl_path}");
    eprintln!("[proxy] spawning: {kubectl_path} {}", args.join(" "));

    // ── spawn; drop the MutexGuard before any .await ─────────────────────────
    {
        let mut guard = state.0.lock().map_err(|e| e.to_string())?;

        if let Some(mut child) = guard.take() {
            eprintln!("[proxy] killing previous proxy process");
            let _ = child.kill();
        }

        let mut child = Command::new(&kubectl_path)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("failed to spawn kubectl proxy: {e}"))?;

        eprintln!("[proxy] spawned pid {:?}", child.id());

        // Print stdout in real time
        if let Some(stdout) = child.stdout.take() {
            thread::spawn(move || {
                for line in BufReader::new(stdout).lines().flatten() {
                    eprintln!("[proxy stdout] {line}");
                }
            });
        }

        // Print stderr in real time
        if let Some(stderr) = child.stderr.take() {
            thread::spawn(move || {
                for line in BufReader::new(stderr).lines().flatten() {
                    eprintln!("[proxy stderr] {line}");
                }
            });
        }

        *guard = Some(child);
    } // MutexGuard dropped here — safe to .await below

    // Fixed 2000 ms sleep while we observe raw proxy output
    eprintln!("[proxy] sleeping 2000ms...");
    sleep(Duration::from_millis(2000)).await;
    eprintln!("[proxy] done sleeping, returning Ok");

    Ok(())
}

/// Kills the running kubectl proxy process.
#[tauri::command]
pub async fn stop_kubectl_proxy(state: State<'_, KubectlProxy>) -> Result<(), String> {
    let mut guard = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(mut child) = guard.take() {
        child.kill().map_err(|e| e.to_string())?;
    }
    Ok(())
}
