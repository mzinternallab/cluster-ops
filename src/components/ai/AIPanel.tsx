// AI analysis sidebar panel — Phase 1 Step 12
export function AIPanel() {
  return (
    <div className="w-[340px] bg-surface border-l border-border flex flex-col shrink-0">
      <div className="flex items-center gap-2 px-4 py-3 border-b border-border">
        <div className="w-2 h-2 rounded-full bg-ai-purple animate-pulse" />
        <span className="text-ai-purple text-xs font-semibold">AI Analysis</span>
      </div>
      <div className="flex-1 p-4 text-text-muted text-xs">
        AI panel — Phase 1 Step 12
      </div>
    </div>
  )
}
