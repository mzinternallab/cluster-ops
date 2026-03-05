// AI analysis commands — all providers handled via AiClient in ai_provider.rs

use tauri::AppHandle;
use crate::ai_provider::{AiClient, AiConfig};

// ── get_ai_provider_name ──────────────────────────────────────────────────────

#[tauri::command]
pub fn get_ai_provider_name() -> String {
    let provider = std::env::var("AI_PROVIDER")
        .unwrap_or_else(|_| "anthropic".to_string())
        .to_lowercase();

    match provider.as_str() {
        "anthropic" => "Ask Claude".to_string(),
        "openai"    => "Ask GPT".to_string(),
        "azure"     => "Ask GPT".to_string(),
        "ollama"    => {
            let model = std::env::var("AI_MODEL")
                .unwrap_or_else(|_| "Ollama".to_string());
            format!("Ask {}", model)
        }
        other => format!("Ask {}", other),
    }
}

// ── ask_ai ────────────────────────────────────────────────────────────────────

#[tauri::command]
pub async fn ask_ai(
    app: AppHandle,
    raw_output: String,
    messages: Vec<serde_json::Value>,
    question: String,
) -> Result<(), String> {
    let config = AiConfig::from_env()?;
    let ai_client = AiClient::new(config);

    // The raw kubectl output is injected as context in the first message.
    let system_context = format!(
        "You are a Kubernetes expert assistant. The user is \
         looking at this kubectl output and has questions about it.\n\n\
         kubectl output:\n{}\n\n\
         Answer questions specifically about this output. \
         Be concise and actionable.",
        raw_output
    );

    let mut full_messages = vec![
        serde_json::json!({ "role": "user", "content": system_context }),
    ];

    // Append conversation history for multi-turn support.
    for msg in &messages {
        full_messages.push(msg.clone());
    }

    // Append the current question.
    full_messages.push(serde_json::json!({
        "role": "user",
        "content": question,
    }));

    ai_client
        .chat_with_events(full_messages, &app, "ask-ai-stream", "ask-ai-done")
        .await
}

#[tauri::command]
pub async fn analyze_with_ai(
    app: AppHandle,
    output: String,
    mode: String, // "describe" or "logs"
) -> Result<(), String> {
    let prompt = if mode == "logs" {
        format!(
            "You are a Kubernetes operations expert. Analyze these pod logs and identify:\n\
             1. Any errors, crashes, panics, or fatal issues\n\
             2. Warnings or concerning patterns\n\
             3. Root cause analysis if possible\n\
             4. Specific actionable kubectl commands to fix issues\n\n\
             Respond ONLY with a JSON object:\n\
             {{\n\
               \"insights\": [\n\
                 {{\n\
                   \"type\": \"critical\" | \"warning\" | \"suggestion\",\n\
                   \"title\": \"Short title\",\n\
                   \"body\": \"Explanation\",\n\
                   \"command\": \"kubectl command if applicable (optional)\"\n\
                 }}\n\
               ]\n\
             }}\n\n\
             Pod logs:\n{output}"
        )
    } else {
        format!(
            "You are a Kubernetes operations expert. Analyze this kubectl describe output and identify:\n\
             1. Any errors, crashes, or critical issues\n\
             2. Warnings or concerning patterns\n\
             3. Specific actionable kubectl commands to fix issues\n\n\
             Respond ONLY with a JSON object:\n\
             {{\n\
               \"insights\": [\n\
                 {{\n\
                   \"type\": \"critical\" | \"warning\" | \"suggestion\",\n\
                   \"title\": \"Short title\",\n\
                   \"body\": \"Explanation\",\n\
                   \"command\": \"kubectl command if applicable (optional)\"\n\
                 }}\n\
               ]\n\
             }}\n\n\
             kubectl describe output:\n{output}"
        )
    };

    let config = AiConfig::from_env()?;
    AiClient::new(config).chat(prompt, &app).await
}

// ── analyze_security ──────────────────────────────────────────────────────────

#[tauri::command]
pub async fn analyze_security(
    app: AppHandle,
    output: String,
) -> Result<(), String> {
    let prompt = format!(
        "You are a Kubernetes security expert specializing in \
US Government security frameworks. Analyze this kubectl \
describe pod output against:\n\
\n\
1. NSA/CISA Kubernetes Hardening Guide (2022)\n\
2. CIS Kubernetes Benchmark v1.8\n\
3. NIST SP 800-190 Container Security\n\
\n\
Check for these specific issues:\n\
- Container running as root (missing runAsNonRoot: true)\n\
- Privileged containers (privileged: true)\n\
- Missing resource limits (CPU and memory)\n\
- hostNetwork, hostPID, or hostIPC set to true\n\
- Writable root filesystem (missing readOnlyRootFilesystem: true)\n\
- Dangerous capabilities (NET_ADMIN, SYS_ADMIN, ALL)\n\
- Missing liveness and readiness probes\n\
- Image using latest tag or no digest pinning\n\
- Auto-mounted service account tokens\n\
- Secrets exposed as environment variables\n\
- Missing seccompProfile\n\
- Missing AppArmor or SELinux profile\n\
\n\
Return ONLY a JSON object:\n\
{{\n\
  \"insights\": [\n\
    {{\n\
      \"type\": \"critical\" | \"warning\" | \"suggestion\",\n\
      \"title\": \"Short title\",\n\
      \"body\": \"Explanation with specific NSA/CISA or CIS control reference\",\n\
      \"command\": \"kubectl or yaml fix command if applicable\"\n\
    }}\n\
  ]\n\
}}\n\
\n\
Return the top findings ordered by severity.\n\
If the pod passes a check, do not include it.\n\
Focus only on actual issues found in the describe output.\n\
\n\
kubectl describe pod output:\n{output}"
    );

    let config = AiConfig::from_env()?;
    AiClient::new(config).chat(prompt, &app).await
}

// ── analyze_network_scan ──────────────────────────────────────────────────────

#[tauri::command]
pub async fn analyze_network_scan(
    app: AppHandle,
    output: String,
) -> Result<(), String> {
    let prompt = format!(
        "You are a Kubernetes network security expert specializing in \
US Government security frameworks (NSA/CISA Kubernetes Hardening \
Guide 2022, NIST SP 800-190).\n\
\n\
Analyze this Kubernetes network configuration data and check for:\n\
- Missing NetworkPolicy for the namespace (no default deny)\n\
- Overly permissive ingress rules (allowing all sources)\n\
- Overly permissive egress rules (allowing all destinations)\n\
- Unrestricted pod-to-pod communication\n\
- Services exposed as LoadBalancer or NodePort unnecessarily\n\
- Ingress without TLS configured\n\
- Missing network segmentation between namespaces\n\
\n\
Return ONLY a JSON object:\n\
{{\n\
  \"insights\": [\n\
    {{\n\
      \"type\": \"critical\" | \"warning\" | \"suggestion\",\n\
      \"title\": \"Short title\",\n\
      \"body\": \"Explanation with NSA/CISA control reference\",\n\
      \"command\": \"kubectl command to fix or investigate\"\n\
    }}\n\
  ]\n\
}}\n\
\n\
Return top findings ordered by severity.\n\
Only report actual issues found in the data.\n\
\n\
Network configuration data:\n{output}"
    );

    let config = AiConfig::from_env()?;
    AiClient::new(config).chat(prompt, &app).await
}

// ── analyze_rbac_scan ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn analyze_rbac_scan(
    app: AppHandle,
    output: String,
) -> Result<(), String> {
    let prompt = format!(
        "You are a Kubernetes RBAC security expert specializing in \
US Government security frameworks (NSA/CISA Kubernetes Hardening \
Guide 2022, NIST SP 800-190, NIST SP 800-53).\n\
\n\
Analyze this Kubernetes RBAC configuration and check for:\n\
- ServiceAccounts with cluster-admin or admin role bindings\n\
- Wildcard permissions (* verbs or * resources) in roles\n\
- Default service account with elevated permissions\n\
- Overly broad role bindings giving namespace-wide access\n\
- Excessive secret access (get/list/watch secrets)\n\
- Ability to exec into pods (pods/exec permission)\n\
- Ability to escalate privileges (escalate, bind, impersonate verbs)\n\
- Roles with delete permissions on critical resources\n\
- ClusterRoleBindings that should be namespace-scoped RoleBindings\n\
\n\
Return ONLY a JSON object:\n\
{{\n\
  \"insights\": [\n\
    {{\n\
      \"type\": \"critical\" | \"warning\" | \"suggestion\",\n\
      \"title\": \"Short title\",\n\
      \"body\": \"Explanation with NIST/NSA control reference\",\n\
      \"command\": \"kubectl command to investigate or remediate\"\n\
    }}\n\
  ]\n\
}}\n\
\n\
Return top findings ordered by severity.\n\
Only report actual issues found in the data.\n\
\n\
RBAC configuration data:\n{output}"
    );

    let config = AiConfig::from_env()?;
    AiClient::new(config).chat(prompt, &app).await
}

// ── analyze_namespace_scan ────────────────────────────────────────────────────

#[tauri::command]
pub async fn analyze_namespace_scan(
    app: AppHandle,
    output: String,
) -> Result<(), String> {
    let prompt = format!(
        "You are a Kubernetes namespace security expert specializing \
in US Government security frameworks (NSA/CISA Kubernetes \
Hardening Guide 2022, NIST SP 800-190, CIS Kubernetes Benchmark).\n\
\n\
Analyze this Kubernetes namespace configuration and check for:\n\
- Missing ResourceQuota (no CPU/memory limits for namespace)\n\
- Missing LimitRange (no default container limits)\n\
- Missing PodSecurity admission labels (enforce, warn, audit)\n\
- No pod disruption budgets defined\n\
- Namespace missing recommended labels and annotations\n\
- Default service account not restricted\n\
- Missing network isolation controls\n\
- Secrets without encryption indicators\n\
- No audit logging configuration visible\n\
\n\
Return ONLY a JSON object:\n\
{{\n\
  \"insights\": [\n\
    {{\n\
      \"type\": \"critical\" | \"warning\" | \"suggestion\",\n\
      \"title\": \"Short title\",\n\
      \"body\": \"Explanation with NSA/CISA or CIS control reference\",\n\
      \"command\": \"kubectl command to fix or investigate\"\n\
    }}\n\
  ]\n\
}}\n\
\n\
Return top findings ordered by severity.\n\
Only report actual issues found in the data.\n\
\n\
Namespace configuration data:\n{output}"
    );

    let config = AiConfig::from_env()?;
    AiClient::new(config).chat(prompt, &app).await
}

// ── analyze_node_scan ─────────────────────────────────────────────────────────

#[tauri::command]
pub async fn analyze_node_scan(
    app: AppHandle,
    output: String,
) -> Result<(), String> {
    let prompt = format!(
        "You are a Kubernetes node security expert specializing in \
US Government security frameworks (NSA/CISA Kubernetes \
Hardening Guide 2022, NIST SP 800-190, CIS Kubernetes Benchmark).\n\
\n\
Analyze this Kubernetes node data and check for:\n\
- Nodes running outdated Kubernetes version\n\
- Nodes under memory or CPU pressure (MemoryPressure, DiskPressure, PIDPressure conditions)\n\
- Unschedulable nodes\n\
- Nodes with too many pods approaching capacity\n\
- Missing node taints for workload isolation\n\
- Nodes without proper labels for workload segregation\n\
- Container runtime version outdated\n\
- Kernel version concerns\n\
- Missing node security configurations\n\
- Pods running on control plane nodes that should not be\n\
- Uneven pod distribution across nodes\n\
\n\
Return ONLY a JSON object:\n\
{{\n\
  \"insights\": [\n\
    {{\n\
      \"type\": \"critical\" | \"warning\" | \"suggestion\",\n\
      \"title\": \"Short title\",\n\
      \"body\": \"Explanation with NSA/CISA or CIS control reference\",\n\
      \"command\": \"kubectl command to fix or investigate\"\n\
    }}\n\
  ]\n\
}}\n\
\n\
Return top findings ordered by severity.\n\
Only report actual issues found in the data.\n\
\n\
Node data:\n{output}"
    );

    let config = AiConfig::from_env()?;
    AiClient::new(config).chat(prompt, &app).await
}
