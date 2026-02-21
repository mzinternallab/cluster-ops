// Send kubectl output to AI and stream response
// Implemented in Phase 1 Step 11-12

export function useAI() {
  // TODO: invoke('analyze_with_ai') + Tauri event stream
  return { analyze: (_output: string, _mode: string) => Promise.resolve(null), insights: [], isLoading: false }
}
