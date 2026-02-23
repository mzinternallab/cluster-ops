use std::collections::{HashMap, HashSet};
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
/// Uses `dirs::home_dir()` for portability (works on Windows, Linux, macOS).
/// On WSL2 also includes the Windows user's `.kube` directory.
fn kube_dirs_to_scan() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // Native home — works on Windows (C:\Users\NAME), Linux (~), and macOS (~)
    if let Some(home) = dirs::home_dir() {
        let d = home.join(".kube");
        eprintln!("[kubeconfig] home kube dir = {}", d.display());
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

// ── file scanning helpers ─────────────────────────────────────────────────────

/// Returns all regular files in `dir` whose name starts with `"config"`, sorted alphabetically.
/// Skips subdirectories and any file whose name does not start with `"config"`.
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
        if !name.starts_with("config") {
            eprintln!("[kubeconfig]   NON-CONFIG (skipped) {}", path.display());
            continue;
        }
        eprintln!("[kubeconfig]   FILE (candidate) {}", path.display());
        paths.push(path);
    }

    paths.sort();
    paths
}

/// Parses each path as a kubeconfig, logging outcomes, and returns successful
/// `(path, Kubeconfig)` pairs.  File provenance is preserved so every context
/// can be associated with the exact file it came from.
fn load_files_from_paths(paths: &[PathBuf]) -> Vec<(PathBuf, Kubeconfig)> {
    paths
        .iter()
        .filter_map(|path| match Kubeconfig::read_from(path) {
            Ok(cfg) => {
                eprintln!(
                    "[kubeconfig]   PARSE ok ({} ctx)  {}",
                    cfg.contexts.len(),
                    path.display()
                );
                Some((path.clone(), cfg))
            }
            Err(e) => {
                eprintln!("[kubeconfig]   PARSE fail ({e})  {}", path.display());
                None
            }
        })
        .collect()
}

// ── context builder ───────────────────────────────────────────────────────────

/// Converts a list of `(file, Kubeconfig)` pairs into `Vec<KubeContext>`.
///
/// - `current_context` is taken from the first file that declares one (mirrors
///   kubectl's resolution order for KUBECONFIG).
/// - The cluster→server map is built across *all* files so cross-file references
///   resolve correctly.
/// - Each `KubeContext` records `kubeconfig_file` — the exact file that contained
///   its context entry — so the caller can pass `--kubeconfig=<file>` directly to
///   kubectl without any multi-path separator issues.
/// - If the same context name appears in multiple files, the first occurrence wins.
fn build_contexts_from_files(files: &[(PathBuf, Kubeconfig)]) -> Vec<KubeContext> {
    // First file with a current_context wins (kubectl behaviour).
    let current = files
        .iter()
        .find_map(|(_, cfg)| cfg.current_context.as_ref())
        .cloned()
        .unwrap_or_default();

    // Cluster → server URL across all files.
    let cluster_servers: HashMap<String, String> = files
        .iter()
        .flat_map(|(_, cfg)| cfg.clusters.iter())
        .filter_map(|nc| {
            let server = nc.cluster.as_ref()?.server.clone()?;
            Some((nc.name.clone(), server))
        })
        .collect();

    // Emit contexts preserving file origin; first occurrence of a name wins.
    let mut seen: HashSet<String> = HashSet::new();
    let mut contexts = Vec::new();

    for (path, cfg) in files {
        for named in &cfg.contexts {
            if !seen.insert(named.name.clone()) {
                continue; // duplicate — already have this context
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
                kubeconfig_file: Some(path.to_string_lossy().into_owned()),
            });
        }
    }

    contexts
}

// ── commands ──────────────────────────────────────────────────────────────────

/// Lists all contexts from the merged kubeconfig.
///
/// Resolution order:
/// 1. If KUBECONFIG env var is set, read each path in it individually (using
///    the platform separator: `;` on Windows, `:` on Unix).  If at least one
///    context is found, return those results with per-context file provenance.
/// 2. Otherwise — KUBECONFIG unset, files missing, or zero contexts — scan
///    every applicable `.kube` directory:
///    - `~/.kube`                     (portable, works on Windows / Linux / macOS)
///    - Windows `%USERPROFILE%\.kube` (converted to `/mnt/…`, WSL2 only)
///    Every regular file whose name starts with `"config"` is attempted.
///    All valid results are returned with per-context file provenance.
#[tauri::command]
pub async fn get_kubeconfig_contexts() -> Result<Vec<KubeContext>, String> {
    let kube_env = std::env::var("KUBECONFIG").unwrap_or_default();
    eprintln!("[kubeconfig] ── get_kubeconfig_contexts ────────────────────────────");
    eprintln!("[kubeconfig] KUBECONFIG env = {:?}", kube_env);
    log::info!("kubeconfig: KUBECONFIG env = {:?}", kube_env);

    // ── 1. Try KUBECONFIG env var — read each file individually ──────────────
    if !kube_env.is_empty() {
        let sep = if cfg!(windows) { ';' } else { ':' };
        let paths: Vec<PathBuf> = kube_env
            .split(sep)
            .filter(|s| !s.trim().is_empty())
            .map(|s| PathBuf::from(s.trim()))
            .collect();

        eprintln!("[kubeconfig] KUBECONFIG env: {} path(s)", paths.len());
        let file_configs = load_files_from_paths(&paths);
        let total: usize = file_configs.iter().map(|(_, c)| c.contexts.len()).sum();

        if total > 0 {
            eprintln!("[kubeconfig] KUBECONFIG env: {total} context(s) — done");
            log::info!("kubeconfig: KUBECONFIG env yielded {total} context(s)");
            let contexts = build_contexts_from_files(&file_configs);
            for c in &contexts {
                eprintln!(
                    "[kubeconfig]   {:?}  active={}  file={:?}",
                    c.name, c.is_active, c.kubeconfig_file
                );
            }
            return Ok(contexts);
        }

        eprintln!("[kubeconfig] KUBECONFIG env: 0 contexts — falling back to dir scan");
        log::warn!("kubeconfig: KUBECONFIG set but yielded 0 contexts; scanning dirs");
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

    eprintln!(
        "[kubeconfig] {} total candidate file(s) across all dirs",
        all_candidates.len()
    );

    let file_configs = load_files_from_paths(&all_candidates);
    if file_configs.is_empty() {
        eprintln!("[kubeconfig] No valid kubeconfig files found — returning empty");
        log::warn!("kubeconfig: no valid kubeconfig files found");
        return Ok(vec![]);
    }

    let contexts = build_contexts_from_files(&file_configs);
    eprintln!("[kubeconfig] Returning {} context(s):", contexts.len());
    for c in &contexts {
        eprintln!(
            "[kubeconfig]   {:?}  active={}  file={:?}",
            c.name, c.is_active, c.kubeconfig_file
        );
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
