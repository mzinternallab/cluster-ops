// AI provider abstraction — supports Anthropic, OpenAI, Azure OpenAI, and Ollama.
// Configuration is read from environment variables at call time.

use tauri::Emitter;

// ── Provider ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub enum AiProvider {
    Anthropic,
    OpenAI,
    Azure,
    Ollama,
}

// ── Config ────────────────────────────────────────────────────────────────────

pub struct AiConfig {
    pub provider:  AiProvider,
    pub api_key:   Option<String>,
    pub model:     String,
    pub base_url:  Option<String>,
}

impl AiConfig {
    /// Build config from environment variables.
    ///
    /// | Variable         | Purpose                                              |
    /// |-----------------|------------------------------------------------------|
    /// | `AI_PROVIDER`   | `anthropic` (default) · `openai` · `azure` · `ollama` |
    /// | `AI_API_KEY`    | API key; falls back to `ANTHROPIC_API_KEY`           |
    /// | `AI_MODEL`      | Model name; sensible default per provider            |
    /// | `AI_BASE_URL`   | Required for azure; optional override for ollama     |
    pub fn from_env() -> Result<Self, String> {
        let provider_str = std::env::var("AI_PROVIDER")
            .unwrap_or_else(|_| "anthropic".to_string())
            .to_lowercase();

        let provider = match provider_str.as_str() {
            "anthropic" => AiProvider::Anthropic,
            "openai"    => AiProvider::OpenAI,
            "azure"     => AiProvider::Azure,
            "ollama"    => AiProvider::Ollama,
            other       => return Err(format!(
                "Unknown AI_PROVIDER '{other}'. Valid values: anthropic, openai, azure, ollama"
            )),
        };

        // API key: AI_API_KEY first, fall back to ANTHROPIC_API_KEY for backwards compat.
        let api_key = std::env::var("AI_API_KEY")
            .ok()
            .or_else(|| std::env::var("ANTHROPIC_API_KEY").ok());

        // Providers that require a key.
        match provider {
            AiProvider::Anthropic | AiProvider::OpenAI | AiProvider::Azure => {
                if api_key.is_none() {
                    return Err(
                        "AI_API_KEY (or ANTHROPIC_API_KEY) must be set for this provider"
                            .to_string(),
                    );
                }
            }
            AiProvider::Ollama => {} // No key required.
        }

        let base_url = std::env::var("AI_BASE_URL").ok();

        // Azure requires a full endpoint URL.
        if matches!(provider, AiProvider::Azure) && base_url.is_none() {
            return Err(
                "AI_BASE_URL is required for the azure provider \
                 (e.g. https://your-resource.openai.azure.com/openai/deployments/\
                 gpt-4/chat/completions?api-version=2024-02-01)"
                    .to_string(),
            );
        }

        // Default model per provider.
        let default_model = match provider {
            AiProvider::Anthropic => "claude-sonnet-4-6",
            AiProvider::OpenAI    => "gpt-4o",
            AiProvider::Azure     => "gpt-4",
            AiProvider::Ollama    => "llama3",
        };
        let model = std::env::var("AI_MODEL").unwrap_or_else(|_| default_model.to_string());

        Ok(AiConfig { provider, api_key, model, base_url })
    }
}

// ── Client ────────────────────────────────────────────────────────────────────

pub struct AiClient {
    config: AiConfig,
    client: reqwest::Client,
}

impl AiClient {
    pub fn new(config: AiConfig) -> Self {
        AiClient { config, client: reqwest::Client::new() }
    }

    /// Send `prompt` to the configured provider and stream the response.
    /// Emits `ai-stream` events for each token and a final `ai-done` event
    /// with the complete accumulated text.
    pub async fn chat(
        &self,
        prompt: String,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        match self.config.provider {
            AiProvider::Anthropic            => self.chat_anthropic(prompt, app).await,
            AiProvider::OpenAI | AiProvider::Azure => self.chat_openai_compat(prompt, app).await,
            AiProvider::Ollama               => self.chat_ollama(prompt, app).await,
        }
    }

    // ── Anthropic ─────────────────────────────────────────────────────────────
    // SSE stream; delta token at data.delta.text

    async fn chat_anthropic(
        &self,
        prompt: String,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let api_key = self.config.api_key.as_deref()
            .ok_or("API key not set")?;

        let body = serde_json::json!({
            "model":      self.config.model,
            "max_tokens": 4096,
            "stream":     true,
            "messages":   [{ "role": "user", "content": prompt }],
        });

        let mut response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Anthropic request failed: {e}"))?;

        let mut buffer = String::new();

        'outer: loop {
            match response.chunk().await.map_err(|e| e.to_string())? {
                None => break,
                Some(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data.trim() == "[DONE]" { break 'outer; }
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(delta) = json["delta"]["text"].as_str() {
                                    buffer.push_str(delta);
                                    app.emit("ai-stream", delta).map_err(|e| e.to_string())?;
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

    // ── OpenAI / Azure ────────────────────────────────────────────────────────
    // SSE stream; delta token at data.choices[0].delta.content

    async fn chat_openai_compat(
        &self,
        prompt: String,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let api_key = self.config.api_key.as_deref()
            .ok_or("API key not set")?;

        // Azure uses the full AI_BASE_URL; OpenAI uses the standard endpoint.
        let url = if matches!(self.config.provider, AiProvider::Azure) {
            self.config.base_url.as_deref()
                .ok_or("AI_BASE_URL is required for azure")?
                .to_string()
        } else {
            "https://api.openai.com/v1/chat/completions".to_string()
        };

        let body = serde_json::json!({
            "model":    self.config.model,
            "stream":   true,
            "messages": [{ "role": "user", "content": prompt }],
        });

        // Azure uses `api-key` header; OpenAI uses Bearer token.
        let request = self.client
            .post(&url)
            .header("content-type", "application/json")
            .json(&body);

        let request = if matches!(self.config.provider, AiProvider::Azure) {
            request.header("api-key", api_key)
        } else {
            request.header("Authorization", format!("Bearer {api_key}"))
        };

        let mut response = request
            .send()
            .await
            .map_err(|e| format!("OpenAI/Azure request failed: {e}"))?;

        let mut buffer = String::new();

        'outer: loop {
            match response.chunk().await.map_err(|e| e.to_string())? {
                None => break,
                Some(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        if let Some(data) = line.strip_prefix("data: ") {
                            if data.trim() == "[DONE]" { break 'outer; }
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
                                if let Some(delta) = json["choices"][0]["delta"]["content"].as_str() {
                                    buffer.push_str(delta);
                                    app.emit("ai-stream", delta).map_err(|e| e.to_string())?;
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

    // ── Ollama ────────────────────────────────────────────────────────────────
    // Newline-delimited JSON stream; token at message.content; done when done==true

    async fn chat_ollama(
        &self,
        prompt: String,
        app: &tauri::AppHandle,
    ) -> Result<(), String> {
        let base = self.config.base_url.as_deref()
            .unwrap_or("http://localhost:11434");
        let url = format!("{base}/api/chat");

        let body = serde_json::json!({
            "model":    self.config.model,
            "stream":   true,
            "messages": [{ "role": "user", "content": prompt }],
        });

        let mut response = self.client
            .post(&url)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {e}"))?;

        let mut buffer = String::new();

        'outer: loop {
            match response.chunk().await.map_err(|e| e.to_string())? {
                None => break,
                Some(chunk) => {
                    let text = String::from_utf8_lossy(&chunk);
                    for line in text.lines() {
                        let line = line.trim();
                        if line.is_empty() { continue; }
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                            if let Some(content) = json["message"]["content"].as_str() {
                                if !content.is_empty() {
                                    buffer.push_str(content);
                                    app.emit("ai-stream", content).map_err(|e| e.to_string())?;
                                }
                            }
                            if json["done"].as_bool().unwrap_or(false) {
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        app.emit("ai-done", &buffer).map_err(|e| e.to_string())?;
        Ok(())
    }
}
