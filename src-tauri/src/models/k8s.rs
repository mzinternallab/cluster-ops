// Rust structs mirroring the TypeScript types in src/types/kubernetes.ts
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KubeContext {
    /// Derived from the kubeconfig filename: "config.eagle-i-orc" → "eagle-i-orc".
    /// Falls back to the context name when the file is just "config".
    /// This is the value shown in the UI.
    pub display_name: String,
    /// The actual context name stored inside the kubeconfig file (e.g. "local").
    /// Always passed as --context to kubectl subprocesses.
    pub context_name: String,
    /// Absolute path of the kubeconfig file that owns this context.
    /// Always passed as --kubeconfig to kubectl subprocesses.
    pub source_file: String,
    pub cluster: String,
    pub user: String,
    pub is_active: bool,
    /// API server URL — used for health checks
    pub server_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PodSummary {
    pub name: String,
    pub namespace: String,
    pub status: String,
    pub ready: String,
    pub restarts: u32,
    pub age: String,
    pub cpu: String,
    pub memory: String,
    pub node: String,
    pub labels: HashMap<String, String>,
}
