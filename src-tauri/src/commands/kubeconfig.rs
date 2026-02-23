use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use kube::config::Kubeconfig;

use crate::models::k8s::KubeContext;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Returns the first kubeconfig file path to use for writes.
/// Respects KUBECONFIG env var (`:` on Unix, `;` on Windows), then ~/.kube/config.
fn primary_kubeconfig_path() -> Option<PathBuf> {
    let sep = if cfg!(windows) { ';' } else { ':' };

    std::env::var("KUBECONFIG")
        .ok()
        .and_then(|v| {
            v.split(sep)
                .next()
                .map(|s| PathBuf::from(s.trim()))
        })
        .or_else(|| dirs::home_dir().map(|h| h.join(".kube").join("config")))
}

// ── commands ──────────────────────────────────────────────────────────────────

/// Lists all contexts found by scanning every file in `~/.kube` whose name
/// starts with `"config"`.  Each returned `KubeContext` carries a `source_file`
/// field recording the exact file it was parsed from.
#[tauri::command]
pub async fn get_kubeconfig_contexts() -> Result<Vec<KubeContext>, String> {
    let kube_dir = dirs::home_dir()
        .map(|h| h.join(".kube"))
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

    println!("Scanning ~/.kube directory: {}", kube_dir.display());

    // ── collect all files whose name starts with "config" ────────────────────
    let mut config_files: Vec<PathBuf> = Vec::new();
    match std::fs::read_dir(&kube_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    continue;
                }
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if name.starts_with("config") {
                    println!("Found file: {name}");
                    config_files.push(path);
                }
            }
        }
        Err(e) => {
            println!("Cannot read ~/.kube directory: {e}");
            return Ok(vec![]);
        }
    }
    config_files.sort();

    // ── parse each file, keeping (path, Kubeconfig) pairs ────────────────────
    let mut parsed: Vec<(PathBuf, Kubeconfig)> = Vec::new();
    for path in &config_files {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        match Kubeconfig::read_from(path) {
            Ok(cfg) => {
                println!("Parsed {} contexts from {filename}", cfg.contexts.len());
                parsed.push((path.clone(), cfg));
            }
            Err(e) => {
                println!("Failed to parse {filename}: {e}");
            }
        }
    }

    if parsed.is_empty() {
        println!("Total contexts found: 0");
        return Ok(vec![]);
    }

    // ── determine active context (first file that declares one wins) ──────────
    let current = parsed
        .iter()
        .find_map(|(_, cfg)| cfg.current_context.as_ref())
        .cloned()
        .unwrap_or_default();

    // ── build cluster → server URL map across all files ───────────────────────
    let cluster_servers: HashMap<String, String> = parsed
        .iter()
        .flat_map(|(_, cfg)| cfg.clusters.iter())
        .filter_map(|nc| {
            let server = nc.cluster.as_ref()?.server.clone()?;
            Some((nc.name.clone(), server))
        })
        .collect();

    // ── emit one KubeContext per unique context name, tagged with source file ──
    let mut seen: HashSet<String> = HashSet::new();
    let mut contexts: Vec<KubeContext> = Vec::new();

    for (path, cfg) in &parsed {
        let source = path.to_string_lossy().into_owned();
        for named in &cfg.contexts {
            if !seen.insert(named.name.clone()) {
                continue; // duplicate name — first file wins
            }
            let Some(ctx) = named.context.as_ref() else {
                continue;
            };
            let server_url = cluster_servers.get(&ctx.cluster).cloned();
            contexts.push(KubeContext {
                name: named.name.clone(),
                cluster: ctx.cluster.clone(),
                user: ctx.user.clone().unwrap_or_default(),
                namespace: ctx.namespace.clone(),
                is_active: named.name == current,
                server_url,
                source_file: Some(source.clone()),
            });
        }
    }

    println!("Total contexts found: {}", contexts.len());
    Ok(contexts)
}

/// Writes the new current-context into the primary kubeconfig file.
/// Preserves all other fields verbatim by parsing as serde_yaml::Value.
#[tauri::command]
pub async fn set_active_context(context_name: String) -> Result<(), String> {
    let path = primary_kubeconfig_path()
        .ok_or_else(|| "Cannot determine kubeconfig path".to_string())?;

    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read kubeconfig: {e}"))?;

    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(&raw).map_err(|e| format!("Failed to parse kubeconfig: {e}"))?;

    doc["current-context"] = serde_yaml::Value::String(context_name);

    let updated = serde_yaml::to_string(&doc)
        .map_err(|e| format!("Failed to serialize kubeconfig: {e}"))?;

    std::fs::write(&path, updated)
        .map_err(|e| format!("Failed to write kubeconfig: {e}"))?;

    Ok(())
}

/// Pings the Kubernetes API server at `<server_url>/healthz` and returns:
/// - "healthy"      — responded in < 1.5 s
/// - "slow"         — responded in 1.5 – 5 s
/// - "unreachable"  — timed out or connection refused
///
/// Accepts invalid / self-signed TLS certs because many k8s clusters use them.
#[tauri::command]
pub async fn check_cluster_health(server_url: String) -> String {
    let client = match reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .timeout(Duration::from_secs(5))
        .build()
    {
        Ok(c) => c,
        Err(_) => return "unreachable".to_string(),
    };

    let url = format!("{}/healthz", server_url.trim_end_matches('/'));
    let started = Instant::now();

    match client.get(&url).send().await {
        Ok(_) => {
            if started.elapsed() > Duration::from_millis(1500) {
                "slow".to_string()
            } else {
                "healthy".to_string()
            }
        }
        Err(_) => "unreachable".to_string(),
    }
}
