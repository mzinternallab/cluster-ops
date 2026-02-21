// Individual AI insight card — Phase 1 Step 12
import type { AIInsight as AIInsightType } from '@/types/ai'

interface AIInsightProps {
  insight: AIInsightType
}

const icons: Record<string, string> = {
  critical: '🔴',
  warning: '🟡',
  suggestion: '💡',
}

export function AIInsight({ insight }: AIInsightProps) {
  return (
    <div className="border border-border rounded-panel p-3 mb-2 text-xs">
      <div className="flex items-center gap-2 mb-1">
        <span>{icons[insight.type]}</span>
        <span className="text-text-primary font-semibold">{insight.title}</span>
      </div>
      <p className="text-text-muted">{insight.body}</p>
      {insight.command && (
        <code className="block mt-2 px-2 py-1 bg-background rounded text-accent text-xxs">
          {insight.command}
        </code>
      )}
    </div>
  )
}
