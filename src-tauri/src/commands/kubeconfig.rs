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

/// In WSL2 the `USERPROFILE` env var is inherited from Windows and looks like
/// `C:\Users\7va`.  This converts it to the equivalent `/mnt/c/Users/7va` path
/// so we can scan the Windows-side `.kube` directory.
///
/// Returns `None` if `USERPROFILE` is unset or does not look like a Windows path.
fn windows_home_in_wsl() -> Option<PathBuf> {
    let profile = std::env::var("USERPROFILE").ok()?;
    let profile = profile.trim();

    // Must start with a drive letter followed by a colon, e.g. "C:"
    let mut chars = profile.chars();
    let drive = chars.next()?.to_ascii_lowercase();
    if chars.next()? != ':' {
        return None;
    }

    // Convert the rest of the path: backslashes → forward slashes
    let rest = profile[2..].replace('\\', "/");
    let wsl_path = format!("/mnt/{drive}{rest}");
    eprintln!("[kubeconfig] Windows home (WSL path) = {wsl_path}");
    Some(PathBuf::from(wsl_path))
}

/// Builds the list of `.kube` directories to scan.
/// Always includes `~/.kube`; also includes the Windows user's `.kube` when
/// running under WSL2 (detected via the USERPROFILE env var).
fn kube_dirs_to_scan() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // WSL / native Linux home
    if let Some(home) = dirs::home_dir() {
        let d = home.join(".kube");
        eprintln!("[kubeconfig] WSL home kube dir = {}", d.display());
        if d.is_dir() {
            dirs.push(d);
        } else {
            eprintln!("[kubeconfig]   (does not exist — skipping)");
        }
    }

    // Windows-side home when running under WSL2
    if let Some(win_home) = windows_home_in_wsl() {
        let d = win_home.join(".kube");
        eprintln!("[kubeconfig] Windows kube dir   = {}", d.display());
        if d.is_dir() {
            dirs.push(d);
        } else {
            eprintln!("[kubeconfig]   (does not exist — skipping)");
        }
    }

    dirs
}

// ── kubeconfig merge helpers ──────────────────────────────────────────────────

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

/// Returns all regular, non-hidden files in `dir`, sorted alphabetically.
/// Skips subdirectories and any file whose name begins with '.'.
fn scan_kube_dir(dir: &std::path::Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("[kubeconfig] ERROR: cannot read {}: {e}", dir.display());
            return paths;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if path.is_dir() {
            eprintln!("[kubeconfig]   DIR  (skipped)    {}", path.display());
            continue;
        }
        if name.starts_with('.') {
            eprintln!("[kubeconfig]   HIDDEN (skipped) {}", path.display());
            continue;
        }
        eprintln!("[kubeconfig]   FILE (candidate) {}", path.display());
        paths.push(path);
    }

    paths.sort();
    paths
}

/// Tries to parse each path as a kubeconfig and merges all that succeed.
/// Every file is logged with its outcome.
fn load_from_paths(paths: &[PathBuf]) -> Option<Kubeconfig> {
    let mut merged: Option<Kubeconfig> = None;

    for path in paths {
        match Kubeconfig::read_from(path) {
            Ok(cfg) => {
                eprintln!(
                    "[kubeconfig]   PARSE ok ({} ctx)  {}",
                    cfg.contexts.len(),
                    path.display()
                );
                merged = Some(match merged.take() {
                    None => cfg,
                    Some(base) => merge_kubeconfig(base, cfg),
                });
            }
            Err(e) => {
                eprintln!("[kubeconfig]   PARSE fail ({e})  {}", path.display());
            }
        }
    }

    merged
}

/// Converts a merged `Kubeconfig` into the `Vec<KubeContext>` the frontend needs.
fn build_contexts(kubeconfig: Kubeconfig) -> Vec<KubeContext> {
    let current = kubeconfig.current_context.clone().unwrap_or_default();

    let cluster_servers: HashMap<String, String> = kubeconfig
        .clusters
        .iter()
        .filter_map(|nc| {
            let server = nc.cluster.as_ref()?.server.clone()?;
            Some((nc.name.clone(), server))
        })
        .collect();

    kubeconfig
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
        .collect()
}

// ── commands ──────────────────────────────────────────────────────────────────

/// Lists all contexts from the merged kubeconfig.
///
/// Resolution order:
/// 1. If KUBECONFIG env var is set AND `Kubeconfig::read()` returns at least one
///    context, use that result (same semantics as kubectl).
/// 2. Otherwise — KUBECONFIG unset, file missing, or yielding zero contexts —
///    scan every `.kube` directory that applies:
///    - `~/.kube`           (WSL/Linux home)
///    - Windows `%USERPROFILE%\.kube`  (converted to `/mnt/…`, WSL2 only)
///    Every regular file in those directories is attempted as a kubeconfig.
///    All valid results are merged.
#[tauri::command]
pub async fn get_kubeconfig_contexts() -> Result<Vec<KubeContext>, String> {
    let kube_env = std::env::var("KUBECONFIG").unwrap_or_default();
    eprintln!("[kubeconfig] ── get_kubeconfig_contexts ────────────────────────────");
    eprintln!("[kubeconfig] KUBECONFIG env = {:?}", kube_env);
    log::info!("kubeconfig: KUBECONFIG env = {:?}", kube_env);

    // ── 1. Try KUBECONFIG env var first ──────────────────────────────────────
    if !kube_env.is_empty() {
        match Kubeconfig::read() {
            Ok(cfg) if !cfg.contexts.is_empty() => {
                eprintln!("[kubeconfig] KUBECONFIG env: found {} context(s) — done", cfg.contexts.len());
                log::info!("kubeconfig: KUBECONFIG env yielded {} context(s)", cfg.contexts.len());
                let contexts = build_contexts(cfg);
                for c in &contexts {
                    eprintln!("[kubeconfig]   {:?}  active={}", c.name, c.is_active);
                }
                return Ok(contexts);
            }
            Ok(_) => {
                eprintln!("[kubeconfig] KUBECONFIG env: 0 contexts — falling back to dir scan");
                log::warn!("kubeconfig: KUBECONFIG set but yielded 0 contexts; scanning dirs");
            }
            Err(e) => {
                eprintln!("[kubeconfig] KUBECONFIG env: read failed ({e}) — falling back to dir scan");
                log::warn!("kubeconfig: KUBECONFIG read failed: {e}; scanning dirs");
            }
        }
    }

    // ── 2. Scan all applicable .kube directories ──────────────────────────────
    let dirs = kube_dirs_to_scan();
    if dirs.is_empty() {
        eprintln!("[kubeconfig] No .kube directories found — returning empty");
        return Ok(vec![]);
    }

    let mut all_candidates: Vec<PathBuf> = Vec::new();
    for dir in &dirs {
        eprintln!("[kubeconfig] Scanning: {}", dir.display());
        let files = scan_kube_dir(dir);
        eprintln!("[kubeconfig]   {} candidate file(s)", files.len());
        all_candidates.extend(files);
    }

    eprintln!("[kubeconfig] {} total candidate file(s) across all dirs", all_candidates.len());

    let Some(merged) = load_from_paths(&all_candidates) else {
        eprintln!("[kubeconfig] No valid kubeconfig files found — returning empty");
        log::warn!("kubeconfig: no valid kubeconfig files found");
        return Ok(vec![]);
    };

    let contexts = build_contexts(merged);
    eprintln!("[kubeconfig] Returning {} context(s):", contexts.len());
    for c in &contexts {
        eprintln!("[kubeconfig]   {:?}  active={}", c.name, c.is_active);
    }
    eprintln!("[kubeconfig] ─────────────────────────────────────────────────────");
    log::info!("kubeconfig: returning {} context(s)", contexts.len());

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
