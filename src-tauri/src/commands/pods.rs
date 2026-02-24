use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{ChildStdin, Command, Stdio};
use std::sync::Mutex;

use chrono::Utc;
use k8s_openapi::api::core::v1::{Namespace as K8sNamespace, Pod};
use kube::{api::{DeleteParams, ListParams}, Api, Client, Config};
use tauri::{AppHandle, Emitter};

use crate::models::k8s::PodSummary;

// Holds the stdin pipe of the running exec session.
// Replaced each time exec_into_pod is called; written by send_exec_input.
static EXEC_STDIN: Mutex<Option<ChildStdin>> = Mutex::new(None);

// ── Client ────────────────────────────────────────────────────────────────────

/// Builds a kube Client that talks to the kubectl proxy on :8001.
/// kubectl proxy handles all auth (exec plugins, aws-iam-authenticator, kubelogin, etc.)
/// so kube-rs never needs to run credential plugins itself.
async fn build_client() -> Result<Client, String> {
    let url: http::Uri = "http://127.0.0.1:8001"
        .parse()
        .map_err(|e| format!("proxy url: {e}"))?;
    let config = Config::new(url);
    Client::try_from(config).map_err(|e| format!("client error: {e}"))
}

// ── Status computation ────────────────────────────────────────────────────────

fn compute_pod_status(pod: &Pod) -> String {
    // Terminating = deletionTimestamp is present, regardless of phase
    if pod.metadata.deletion_timestamp.is_some() {
        return "Terminating".to_string();
    }

    let status = pod.status.as_ref();

    // Walk container statuses for specific waiting/terminated reasons
    if let Some(css) = status.and_then(|s| s.container_statuses.as_ref()) {
        for cs in css {
            if let Some(state) = &cs.state {
                // Check waiting reason first (CrashLoopBackOff, ImagePullBackOff, …)
                if let Some(waiting) = &state.waiting {
                    if let Some(reason) = &waiting.reason {
                        match reason.as_str() {
                            "CrashLoopBackOff"
                            | "ImagePullBackOff"
                            | "ErrImagePull"
                            | "CreateContainerConfigError"
                            | "InvalidImageName" => return reason.clone(),
                            _ => {}
                        }
                    }
                }
                // Check terminated reason (OOMKilled, Error, …)
                if let Some(term) = &state.terminated {
                    if let Some(reason) = &term.reason {
                        match reason.as_str() {
                            "OOMKilled" | "Error" | "Completed" => return reason.clone(),
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    // Fall back to pod phase
    status
        .and_then(|s| s.phase.as_deref())
        .unwrap_or("Unknown")
        .to_string()
}

// ── Age formatting ────────────────────────────────────────────────────────────

fn format_age(ts: &k8s_openapi::apimachinery::pkg::apis::meta::v1::Time) -> String {
    let elapsed = Utc::now().signed_duration_since(ts.0);
    let secs = elapsed.num_seconds().max(0);

    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3_600 {
        format!("{}m", secs / 60)
    } else if secs < 86_400 {
        format!("{}h", secs / 3_600)
    } else if secs < 86_400 * 365 {
        format!("{}d", secs / 86_400)
    } else {
        format!("{}y", secs / (86_400 * 365))
    }
}

// ── Pod → PodSummary ──────────────────────────────────────────────────────────

fn pod_to_summary(pod: Pod) -> PodSummary {
    let meta = &pod.metadata;
    let spec = pod.spec.as_ref();
    let status = pod.status.as_ref();

    let name = meta.name.clone().unwrap_or_default();
    let namespace = meta.namespace.clone().unwrap_or_default();

    // Ready: "<ready_count>/<total_containers>"
    let total = spec.map(|s| s.containers.len()).unwrap_or(0);
    let ready_count = status
        .and_then(|s| s.container_statuses.as_ref())
        .map(|css| css.iter().filter(|cs| cs.ready).count())
        .unwrap_or(0);
    let ready = format!("{ready_count}/{total}");

    // Restarts: sum across all containers
    let restarts: u32 = status
        .and_then(|s| s.container_statuses.as_ref())
        .map(|css| css.iter().map(|cs| cs.restart_count.max(0) as u32).sum())
        .unwrap_or(0);

    // Age
    let age = meta
        .creation_timestamp
        .as_ref()
        .map(format_age)
        .unwrap_or_else(|| "unknown".to_string());

    // Node
    let node = spec
        .and_then(|s| s.node_name.clone())
        .unwrap_or_default();

    // Labels — BTreeMap → HashMap for JSON serialisation
    let labels: HashMap<String, String> = meta
        .labels
        .clone()
        .unwrap_or_default()
        .into_iter()
        .collect();

    PodSummary {
        status: compute_pod_status(&pod),
        name,
        namespace,
        ready,
        restarts,
        age,
        cpu: "N/A".to_string(),    // metrics-server — Phase 2
        memory: "N/A".to_string(), // metrics-server — Phase 2
        node,
        labels,
    }
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Lists pods in `namespace`, or all namespaces when `namespace` is None / empty.
#[tauri::command]
pub async fn list_pods(namespace: Option<String>) -> Result<Vec<PodSummary>, String> {
    let client = build_client().await?;

    let pods = match namespace.as_deref().filter(|s| !s.is_empty()) {
        Some(ns) => {
            let api: Api<Pod> = Api::namespaced(client, ns);
            api.list(&ListParams::default())
                .await
                .map_err(|e| e.to_string())?
                .items
        }
        None => {
            let api: Api<Pod> = Api::all(client);
            api.list(&ListParams::default())
                .await
                .map_err(|e| e.to_string())?
                .items
        }
    };

    Ok(pods.into_iter().map(pod_to_summary).collect())
}

/// Lists all namespace names in the active cluster.
#[tauri::command]
pub async fn list_namespaces() -> Result<Vec<String>, String> {
    let client = build_client().await?;
    let api: Api<K8sNamespace> = Api::all(client);

    let ns_list = api
        .list(&ListParams::default())
        .await
        .map_err(|e| e.to_string())?;

    let mut names: Vec<String> = ns_list
        .items
        .into_iter()
        .filter_map(|ns| ns.metadata.name)
        .collect();

    names.sort();
    Ok(names)
}

/// Deletes a pod by name and namespace using kube-rs.
#[tauri::command]
pub async fn delete_pod(name: String, namespace: String) -> Result<(), String> {
    let client = build_client().await?;
    let api: Api<Pod> = Api::namespaced(client, &namespace);
    api.delete(&name, &DeleteParams::default())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Execs into a pod via kubectl with a pipe-based interactive shell.
///
/// Spawns `kubectl exec -i … -- /bin/sh` with stdin/stdout/stderr all piped.
/// Immediately writes PS1 setup and an `echo ready` probe to stdin so the shell
/// produces visible output without a PTY.  Stdout and stderr are streamed
/// line-by-line to the frontend via Tauri events.  The frontend sends keystrokes
/// back via `send_exec_input`.
///
/// Events emitted:
/// - `exec-output` — payload: String — one stdout line
/// - `exec-error`  — payload: String — one stderr line
/// - `exec-done`   — payload: null   — session ended
#[tauri::command]
pub async fn exec_into_pod(
    app: AppHandle,
    name: String,
    namespace: String,
    source_file: String,
    context_name: String,
) -> Result<(), String> {
    let kubectl = which::which("kubectl")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "kubectl".to_string());

    eprintln!(
        "[exec] {kubectl} exec -i {name} -n {namespace} \
         --kubeconfig={source_file} --context={context_name} -- /bin/sh"
    );

    let mut child = Command::new(&kubectl)
        .args([
            "exec", "-i", &name, "-n", &namespace,
            &format!("--kubeconfig={source_file}"),
            &format!("--context={context_name}"),
            "--", "/bin/sh",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("kubectl not found: {e}"))?;

    // Write init commands before storing stdin so the shell produces a prompt.
    let mut stdin = child.stdin.take().ok_or("no stdin")?;
    stdin.write_all(b"export PS1='$ '\n").map_err(|e| format!("stdin init: {e}"))?;
    stdin.write_all(b"echo ready\n").map_err(|e| format!("stdin init: {e}"))?;
    stdin.flush().map_err(|e| format!("stdin flush: {e}"))?;

    // Store stdin so send_exec_input can forward keystrokes to the shell.
    {
        let mut guard = EXEC_STDIN.lock().map_err(|e| e.to_string())?;
        *guard = Some(stdin);
    }

    let stdout = child.stdout.take().ok_or("no stdout")?;
    let stderr = child.stderr.take().ok_or("no stderr")?;

    // Stream stdout in a blocking thread — emit each line as an event.
    let app_stdout = app.clone();
    let stdout_task = tokio::task::spawn_blocking(move || {
        for line in BufReader::new(stdout).lines().flatten() {
            let _ = app_stdout.emit("exec-output", line);
        }
    });

    // Stream stderr in a blocking thread — emit each line as an event.
    let app_stderr = app.clone();
    let stderr_task = tokio::task::spawn_blocking(move || {
        for line in BufReader::new(stderr).lines().flatten() {
            let _ = app_stderr.emit("exec-error", line);
        }
    });

    // Once both readers finish (process exited), signal the frontend.
    tokio::spawn(async move {
        let _ = tokio::join!(stdout_task, stderr_task);
        let _ = app.emit("exec-done", ());
    });

    Ok(())
}

/// Forwards input from xterm.js to the running exec session's stdin pipe.
/// Strips \r so Windows CRLF never reaches the shell; each chunk is written
/// followed by \n so the shell processes it as a complete line.
#[tauri::command]
pub async fn send_exec_input(input: String) -> Result<(), String> {
    let clean_input = input.replace('\r', "\n");
    let mut guard = EXEC_STDIN.lock().map_err(|e| e.to_string())?;
    if let Some(ref mut stdin) = *guard {
        stdin.write_all(clean_input.as_bytes()).map_err(|e| format!("stdin write: {e}"))?;
        stdin.write_all(b"\n").map_err(|e| format!("stdin newline: {e}"))?;
        stdin.flush().map_err(|e| format!("stdin flush: {e}"))?;
    }
    Ok(())
}
