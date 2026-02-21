// Rust structs mirroring the TypeScript types in src/types/kubernetes.ts
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KubeContext {
    pub name: String,
    pub cluster: String,
    pub user: String,
    pub namespace: Option<String>,
    pub is_active: bool,
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
