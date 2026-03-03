// Anthropic Claude API integration — Phase 1 Step 11 + security scan
// Model: claude-sonnet-4-6
// Transport: reqwest + SSE streaming

use tauri::{AppHandle, Emitter};

#[tauri::command]
pub async fn analyze_with_ai(
    app: AppHandle,
    output: String,
    mode: String, // "describe" or "logs"
) -> Result<(), String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?;

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

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 4096,
        "stream": true,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let mut response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {e}"))?;

    let mut buffer = String::new();
    let mut done = false;

    while !done {
        match response.chunk().await.map_err(|e| e.to_string())? {
            None => break,
            Some(chunk) => {
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data.trim() == "[DONE]" {
                            done = true;
                            break;
                        }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(delta) = json["delta"]["text"].as_str() {
                                buffer.push_str(delta);
                                app.emit("ai-stream", delta)
                                    .map_err(|e| e.to_string())?;
                            }
                        }
                    }
                }
            }
        }
    }

    app.emit("ai-done", &buffer).map_err(|e| e.to_string())?;
    Ok(())
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

    stream_ai_response(app, prompt).await
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

    stream_ai_response(app, prompt).await
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

    stream_ai_response(app, prompt).await
}

// ── shared SSE streaming helper ───────────────────────────────────────────────

async fn stream_ai_response(app: AppHandle, prompt: String) -> Result<(), String> {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?;

    let client = reqwest::Client::new();

    let body = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "max_tokens": 4096,
        "stream": true,
        "messages": [
            {"role": "user", "content": prompt}
        ]
    });

    let mut response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("API request failed: {e}"))?;

    let mut buffer = String::new();
    let mut done = false;

    while !done {
        match response.chunk().await.map_err(|e| e.to_string())? {
            None => break,
            Some(chunk) => {
                let text = String::from_utf8_lossy(&chunk);
                for line in text.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        if data.trim() == "[DONE]" {
                            done = true;
                            break;
                        }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                            if let Some(delta) = json["delta"]["text"].as_str() {
                                buffer.push_str(delta);
                                app.emit("ai-stream", delta)
                                    .map_err(|e| e.to_string())?;
                            }
                        }
                    }
                }
            }
        }
    }

    app.emit("ai-done", &buffer).map_err(|e| e.to_string())?;
    Ok(())
}
