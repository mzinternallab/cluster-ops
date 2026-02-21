// List/get pods via kube-rs — Phase 1 Step 6
use crate::models::k8s::PodSummary;

/// Lists all pods in the given namespace (or all namespaces if None).
#[tauri::command]
pub async fn list_pods(_namespace: Option<String>) -> Result<Vec<PodSummary>, String> {
    // TODO: implement with kube-rs in Phase 1 Step 6
    Ok(vec![])
}

/// Lists all namespaces in the active cluster.
#[tauri::command]
pub async fn list_namespaces() -> Result<Vec<String>, String> {
    // TODO: implement with kube-rs in Phase 1 Step 6
    Ok(vec![])
}
