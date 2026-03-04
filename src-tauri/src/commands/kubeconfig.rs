use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use kube::config::Kubeconfig;

use crate::models::k8s::KubeContext;

// ── path helpers ──────────────────────────────────────────────────────────────

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

/// Derives a display name from a `config.*` filename suffix.
/// "config.eagle-i-orc" → "eagle-i-orc", "config" → None
fn suffix_from_filename(filename: &str) -> Option<&str> {
    filename
        .strip_prefix("config.")
        .filter(|s| !s.is_empty())
}

// ── commands ──────────────────────────────────────────────────────────────────

/// Lists all contexts found by scanning `~/.kube`:
///
/// 1. `~/.kube/config` (merged config) — contexts are used as-is; display
///    name is the context name unless it is "local" or "default", in which
///    case it falls back to the filename suffix (always "config" → no suffix,
///    so it stays as the context name).
///
/// 2. `~/.kube/config.*` files — if context name is "local", display name is
///    derived from the filename suffix; otherwise the context name is used.
///
/// Contexts are deduplicated by context name across all sources — the first
/// occurrence wins (merged config is processed first).
#[tauri::command]
pub async fn get_kubeconfig_contexts() -> Result<Vec<KubeContext>, String> {
    let kube_dir = dirs::home_dir()
        .map(|h| h.join(".kube"))
        .ok_or_else(|| "Cannot determine home directory".to_string())?;

    // ── (a) merged config ─────────────────────────────────────────────────────
    // Parsed first so its contexts win deduplication.
    let merged_path = kube_dir.join("config");
    let mut parsed: Vec<(PathBuf, bool)> = Vec::new(); // (path, is_merged)

    if merged_path.is_file() {
        if Kubeconfig::read_from(&merged_path).is_ok() {
            parsed.push((merged_path.clone(), true));
        }
    }

    // ── (b) individual config.* files ────────────────────────────────────────
    let mut individual_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&kube_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with("config.") {
                individual_files.push(path);
            }
        }
    }
    individual_files.sort();
    for path in individual_files {
        parsed.push((path, false));
    }

    if parsed.is_empty() {
        return Ok(vec![]);
    }

    // ── parse each file, keeping (path, Kubeconfig, is_merged) triples ───────
    let mut all: Vec<(PathBuf, Kubeconfig, bool)> = Vec::new();
    for (path, is_merged) in &parsed {
        if let Ok(cfg) = Kubeconfig::read_from(path) {
            all.push((path.clone(), cfg, *is_merged));
        }
    }

    if all.is_empty() {
        return Ok(vec![]);
    }

    // ── determine which (source_file, context_name) pair is active ────────────
    let (active_source, active_ctx_name) = all
        .iter()
        .find_map(|(path, cfg, _)| {
            cfg.current_context
                .as_ref()
                .map(|c| (path.to_string_lossy().into_owned(), c.clone()))
        })
        .unwrap_or_default();

    // ── build cluster → server URL map across all files ───────────────────────
    let cluster_servers: HashMap<String, String> = all
        .iter()
        .flat_map(|(_, cfg, _)| cfg.clusters.iter())
        .filter_map(|nc| {
            let server = nc.cluster.as_ref()?.server.clone()?;
            Some((nc.name.clone(), server))
        })
        .collect();

    // ── emit one KubeContext per context, deduplicating by context name ───────
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut contexts: Vec<KubeContext> = Vec::new();

    for (path, cfg, is_merged) in &all {
        let source = path.to_string_lossy().into_owned();
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        for named in &cfg.contexts {
            let Some(ctx) = named.context.as_ref() else {
                continue;
            };
            let context_name = named.name.clone();

            // First occurrence of a context name wins.
            if !seen.insert(context_name.clone()) {
                continue;
            }

            let display_name = if *is_merged {
                // Merged config: context names are already descriptive
                // (e.g. "eagle-i-orc", "rovi"). Use directly.
                context_name.clone()
            } else {
                // Individual config.* file: if context name is "local",
                // derive display name from filename suffix.
                if context_name == "local" {
                    suffix_from_filename(filename)
                        .unwrap_or(&context_name)
                        .to_string()
                } else {
                    context_name.clone()
                }
            };

            let server_url = cluster_servers.get(&ctx.cluster).cloned();
            let is_active = source == active_source && context_name == active_ctx_name;

            contexts.push(KubeContext {
                display_name,
                context_name,
                source_file: source.clone(),
                cluster: ctx.cluster.clone(),
                user: ctx.user.clone().unwrap_or_default(),
                is_active,
                server_url,
            });
        }
    }

    Ok(contexts)
}

/// Writes the new current-context into the source file for the selected context.
/// Preserves all other fields verbatim by parsing as serde_yaml::Value.
#[tauri::command]
pub async fn set_active_context(
    context_name: String,
    source_file: Option<String>,
) -> Result<(), String> {
    // Write to the specific source file if provided; otherwise fall back to
    // the primary kubeconfig path.
    let path = source_file
        .map(PathBuf::from)
        .or_else(primary_kubeconfig_path)
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
