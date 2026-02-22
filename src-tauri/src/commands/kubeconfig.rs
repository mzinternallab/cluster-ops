use std::collections::HashMap;
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

/// Merges `extra` into `base` by extending clusters, auth_infos, and contexts.
/// `base.current_context` wins; `extra.current_context` is used only if base has none.
fn merge_kubeconfig(mut base: Kubeconfig, extra: Kubeconfig) -> Kubeconfig {
    base.clusters.extend(extra.clusters);
    base.auth_infos.extend(extra.auth_infos);
    base.contexts.extend(extra.contexts);
    if base.current_context.is_none() {
        base.current_context = extra.current_context;
    }
    base
}

/// Loads and merges kubeconfig files from an explicit list of paths.
/// Paths that do not exist are silently skipped and logged.
fn load_from_paths(paths: &[PathBuf]) -> Option<Kubeconfig> {
    let mut merged: Option<Kubeconfig> = None;
    for path in paths {
        if !path.exists() {
            log::warn!("kubeconfig: file not found — {}", path.display());
            continue;
        }
        match Kubeconfig::read_from(path) {
            Ok(cfg) => {
                log::info!("kubeconfig: loaded {} context(s) from {}", cfg.contexts.len(), path.display());
                merged = Some(match merged.take() {
                    None => cfg,
                    Some(base) => merge_kubeconfig(base, cfg),
                });
            }
            Err(e) => log::warn!("kubeconfig: failed to parse {}: {e}", path.display()),
        }
    }
    merged
}

// ── commands ──────────────────────────────────────────────────────────────────

/// Lists all contexts from the merged kubeconfig.
///
/// Resolution order:
/// 1. If KUBECONFIG env var is set, delegate to `Kubeconfig::read()` which merges
///    every file in the list (same semantics as kubectl).
/// 2. If KUBECONFIG is unset (common when the Tauri process does not inherit the
///    shell environment, e.g. launched from a desktop icon on Windows/macOS),
///    manually load and merge the two known config files:
///    ~/.kube/config.eagle-i-orc and ~/.kube/config.rovi
///
/// Returns an empty vec — not an error — when no kubeconfig can be found.
#[tauri::command]
pub async fn get_kubeconfig_contexts() -> Result<Vec<KubeContext>, String> {
    let kube_env = std::env::var("KUBECONFIG").unwrap_or_default();
    log::info!("kubeconfig: KUBECONFIG env = {:?}", kube_env);

    let kubeconfig = if !kube_env.is_empty() {
        // Env var is set — kube-rs read() merges all listed files
        match Kubeconfig::read() {
            Ok(cfg) => {
                log::info!(
                    "kubeconfig: Kubeconfig::read() found {} context(s)",
                    cfg.contexts.len()
                );
                cfg
            }
            Err(e) => {
                log::warn!("kubeconfig: Kubeconfig::read() failed: {e}");
                return Ok(vec![]);
            }
        }
    } else {
        // Env var not set — manually load the known config files
        log::info!("kubeconfig: KUBECONFIG not set; loading known files manually");
        let home = match dirs::home_dir() {
            Some(h) => h,
            None => {
                log::warn!("kubeconfig: cannot determine home directory");
                return Ok(vec![]);
            }
        };
        let paths = vec![
            home.join(".kube/config.eagle-i-orc"),
            home.join(".kube/config.rovi"),
        ];
        match load_from_paths(&paths) {
            Some(cfg) => cfg,
            None => {
                log::warn!("kubeconfig: no config files found in fallback paths");
                return Ok(vec![]);
            }
        }
    };

    let ctx_count = kubeconfig.contexts.len();
    log::info!("kubeconfig: {ctx_count} total context(s) after merge");

    let current = kubeconfig.current_context.clone().unwrap_or_default();

    // Build a cluster-name → server-URL lookup from the clusters stanza
    let cluster_servers: HashMap<String, String> = kubeconfig
        .clusters
        .iter()
        .filter_map(|nc| {
            let server = nc.cluster.as_ref()?.server.clone()?;
            Some((nc.name.clone(), server))
        })
        .collect();

    let contexts = kubeconfig
        .contexts
        .into_iter()
        .filter_map(|named| {
            let ctx = named.context?;
            let server_url = cluster_servers.get(&ctx.cluster).cloned();
            Some(KubeContext {
                name: named.name.clone(),
                cluster: ctx.cluster,
                user: ctx.user.unwrap_or_default(),
                namespace: ctx.namespace,
                is_active: named.name == current,
                server_url,
            })
        })
        .collect();

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
