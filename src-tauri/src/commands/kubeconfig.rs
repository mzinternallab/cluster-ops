// List clusters from kubeconfig — Phase 1 Step 4
// Uses kube-rs to parse ~/.kube/config and KUBECONFIG env var

use crate::models::k8s::KubeContext;

/// Returns all available kubeconfig contexts.
#[tauri::command]
pub async fn get_kubeconfig_contexts() -> Result<Vec<KubeContext>, String> {
    // TODO: implement with kube-rs in Phase 1 Step 4
    Ok(vec![])
}

/// Sets the active kubeconfig context.
#[tauri::command]
pub async fn set_active_context(_context_name: String) -> Result<(), String> {
    // TODO: implement in Phase 1 Step 5
    Ok(())
}
