use std::path::PathBuf;

use kube::config::Kubeconfig;

use crate::models::k8s::KubeContext;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Returns the first kubeconfig file path that should be used for writes.
/// Respects the KUBECONFIG env var (colon-separated on Unix, semi-colon on Windows),
/// falling back to ~/.kube/config.
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

/// Lists all contexts from the merged kubeconfig (KUBECONFIG env + ~/.kube/config).
/// Returns an empty vec (not an error) when no kubeconfig exists.
#[tauri::command]
pub async fn get_kubeconfig_contexts() -> Result<Vec<KubeContext>, String> {
    let kubeconfig = match Kubeconfig::read() {
        Ok(cfg) => cfg,
        // No kubeconfig at all — return empty list rather than an error
        Err(_) => return Ok(vec![]),
    };

    let current = kubeconfig.current_context.unwrap_or_default();

    let contexts = kubeconfig
        .contexts
        .into_iter()
        .filter_map(|named| {
            let ctx = named.context?;
            Some(KubeContext {
                name: named.name.clone(),
                cluster: ctx.cluster,
                user: ctx.user.unwrap_or_default(),
                namespace: ctx.namespace,
                is_active: named.name == current,
            })
        })
        .collect();

    Ok(contexts)
}

/// Writes the new current-context into the primary kubeconfig file.
/// Preserves all other fields verbatim.
#[tauri::command]
pub async fn set_active_context(context_name: String) -> Result<(), String> {
    let path = primary_kubeconfig_path()
        .ok_or_else(|| "Cannot determine kubeconfig path".to_string())?;

    let raw = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read kubeconfig: {e}"))?;

    // Use serde_yaml::Value to preserve unknown fields / ordering
    let mut doc: serde_yaml::Value =
        serde_yaml::from_str(&raw).map_err(|e| format!("Failed to parse kubeconfig: {e}"))?;

    doc["current-context"] = serde_yaml::Value::String(context_name);

    let updated = serde_yaml::to_string(&doc)
        .map_err(|e| format!("Failed to serialize kubeconfig: {e}"))?;

    std::fs::write(&path, updated)
        .map_err(|e| format!("Failed to write kubeconfig: {e}"))?;

    Ok(())
}
