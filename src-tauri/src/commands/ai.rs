// Call Anthropic Claude API and stream response — Phase 1 Step 11
// Model: claude-sonnet-4-6
// Transport: reqwest + SSE streaming
// API key: OS keychain via tauri-plugin-keyring

/// Analyzes kubectl output with Claude AI.
/// Streams response tokens via Tauri events.
/// mode: "describe" | "logs"
#[tauri::command]
pub async fn analyze_with_ai(_output: String, _mode: String) -> Result<String, String> {
    // TODO: implement in Phase 1 Step 11
    Ok(String::new())
}
