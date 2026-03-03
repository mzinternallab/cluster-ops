AiProvider enum — Anthropic | OpenAI | Azure | Ollama
AiConfig struct — provider, api_key, model, base_url
AiClient struct — wraps reqwest, handles differences

Methods:
- AiConfig::from_env() — reads env vars, returns config
- AiClient::new(config) — builds client
- AiClient::chat(prompt) -> Stream — sends request, streams response
  handles different request/response formats per provider
