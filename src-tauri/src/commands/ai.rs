// Anthropic Claude API integration — Phase 1 Step 11
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
        "max_tokens": 1024,
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
