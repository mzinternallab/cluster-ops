// Streaming loader animation for AI responses — Phase 1 Step 12
export function AILoader() {
  return (
    <div className="flex items-center gap-2 p-4 text-ai-purple text-xs">
      <div className="flex gap-1">
        {[0, 1, 2].map((i) => (
          <div
            key={i}
            className="w-1.5 h-1.5 rounded-full bg-ai-purple animate-bounce"
            style={{ animationDelay: `${i * 0.15}s` }}
          />
        ))}
      </div>
      <span>Analyzing...</span>
    </div>
  )
}
