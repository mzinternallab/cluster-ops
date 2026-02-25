// AI analysis sidebar panel — Phase 1 Step 12

import { useCallback, useEffect, useRef, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { Check, Copy, RefreshCw } from 'lucide-react'

import { cn } from '@/lib/utils'
import type { AIAnalysisResponse, AIInsight } from '@/types/ai'

// ── InsightCard ───────────────────────────────────────────────────────────────

function InsightCard({ insight }: { insight: AIInsight }) {
  const [copied, setCopied] = useState(false)

  const borderColor = {
    critical:   'border-l-red-500',
    warning:    'border-l-yellow-400',
    suggestion: 'border-l-[#7a7adc]',
  }[insight.type] ?? 'border-l-border'

  const icon = {
    critical:   '🔴',
    warning:    '🟡',
    suggestion: '💡',
  }[insight.type] ?? '•'

  const copy = async () => {
    if (!insight.command) return
    await navigator.clipboard.writeText(insight.command)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  return (
    <div className={cn('border-l-2 pl-3 py-2 mb-3', borderColor)}>
      <div className="flex items-center gap-1.5 mb-1">
        <span className="text-xs leading-none">{icon}</span>
        <span className="text-xs font-semibold text-text-primary">{insight.title}</span>
      </div>
      <p className="text-xs text-text-muted leading-relaxed">{insight.body}</p>
      {insight.command && (
        <div className="mt-2 flex items-center gap-2 bg-background rounded px-2 py-1.5">
          <code className="text-xxs font-mono text-accent flex-1 min-w-0 truncate">
            {insight.command}
          </code>
          <button
            onClick={copy}
            className="shrink-0 text-text-muted hover:text-text-primary transition-colors"
            title="Copy command"
          >
            {copied ? <Check size={11} /> : <Copy size={11} />}
          </button>
        </div>
      )}
    </div>
  )
}

// ── AIPanel ───────────────────────────────────────────────────────────────────

interface AIPanelProps {
  output: string
  mode: 'describe' | 'logs'
  analyzeKey?: number
  onAnalyze?: () => void
  onStreamingChange?: (streaming: boolean) => void
}

export function AIPanel({ output, mode, analyzeKey = 0, onAnalyze, onStreamingChange }: AIPanelProps) {
  const [streaming, setStreaming] = useState(false)
  const [insights, setInsights]   = useState<AIInsight[]>([])
  const [error, setError]         = useState<string | null>(null)

  const activeRef      = useRef(false)
  const unlistensRef   = useRef<(() => void)[]>([])
  const analyzedRef    = useRef('')
  const analyzedKeyRef = useRef(-1)

  // ── Listener cleanup ─────────────────────────────────────────────────────

  const stopListeners = useCallback(() => {
    activeRef.current = false
    unlistensRef.current.forEach((fn) => fn())
    unlistensRef.current = []
  }, [])

  // ── Core analysis runner ─────────────────────────────────────────────────

  const runAnalysis = useCallback(async (out: string, m: string) => {
    stopListeners()
    activeRef.current = true
    analyzedRef.current = out

    setStreaming(true)
    setInsights([])
    setError(null)

    try {
      const uls = await Promise.all([
        // Tokens stream in; we render the full JSON on ai-done
        listen<string>('ai-stream', () => {}),
        listen<string>('ai-done', (e) => {
          if (!activeRef.current) return
          stopListeners()
          setStreaming(false)
          const extractJson = (raw: string): string => {
            // Try to extract from markdown code fence first
            const fenceMatch = raw.match(/```(?:json)?\s*([\s\S]*?)```/)
            if (fenceMatch) return fenceMatch[1].trim()
            // Try to find raw JSON object
            const jsonMatch = raw.match(/\{[\s\S]*\}/)
            if (jsonMatch) return jsonMatch[0].trim()
            // Return as-is and let JSON.parse fail with a useful error
            return raw.trim()
          }
          const cleanJson = extractJson(e.payload)
          try {
            const parsed: AIAnalysisResponse = JSON.parse(cleanJson)
            setInsights(parsed.insights ?? [])
          } catch (err) {
            console.error('[ai] parse failed. raw:', e.payload)
            setError('Failed to parse AI response — check console for details')
          }
        }),
      ])

      if (!activeRef.current) {
        uls.forEach((fn) => fn())
        return
      }
      unlistensRef.current = uls

      await invoke('analyze_with_ai', { output: out, mode: m })
    } catch (err: unknown) {
      if (!activeRef.current) return
      stopListeners()
      setStreaming(false)
      setError(String(err))
    }
  }, [stopListeners])

  // ── Notify parent of streaming state changes ─────────────────────────────

  useEffect(() => {
    onStreamingChange?.(streaming)
  }, [streaming, onStreamingChange])

  // ── Run analysis when output or analyzeKey changes ───────────────────────

  useEffect(() => {
    if (output && (output !== analyzedRef.current || analyzeKey !== analyzedKeyRef.current)) {
      analyzedKeyRef.current = analyzeKey
      runAnalysis(output, mode)
    }
  }, [output, mode, runAnalysis, analyzeKey])

  // ── Reset insights when mode changes ────────────────────────────────────

  useEffect(() => {
    stopListeners()
    analyzedRef.current = ''
    analyzedKeyRef.current = -1
    setInsights([])
    setError(null)
    setStreaming(false)
  }, [mode, stopListeners])

  // ── Unmount cleanup ──────────────────────────────────────────────────────

  useEffect(() => () => stopListeners(), [stopListeners])

  // ── Re-analyze handler (internal header button) ───────────────────────────

  const handleReanalyze = () => {
    if (!output || streaming) return
    analyzedRef.current = ''
    runAnalysis(output, mode)
  }

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div
      className="w-[340px] flex flex-col shrink-0 border-l border-border"
      style={{ background: '#080c14' }}
    >
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2.5 border-b border-border shrink-0">
        <div className="flex items-center gap-2">
          <span className="text-[#7a7adc] text-sm leading-none">✦</span>
          <span className="text-[#7a7adc] text-xs font-semibold tracking-wide">
            AI Analysis
          </span>
        </div>
        {!streaming && output && (
          <button
            onClick={handleReanalyze}
            className="flex items-center gap-1 text-xxs text-text-muted hover:text-text-primary transition-colors"
            title="Re-analyze"
          >
            <RefreshCw size={11} />
            Re-analyze
          </button>
        )}
      </div>

      {/* Body */}
      <div className="flex-1 overflow-y-auto p-4">

        {/* Loading dots */}
        {streaming && (
          <div className="flex items-center gap-2 text-[#7a7adc] text-xs mb-4">
            <div className="flex gap-1">
              {[0, 1, 2].map((i) => (
                <div
                  key={i}
                  className="w-1.5 h-1.5 rounded-full bg-[#7a7adc] animate-bounce"
                  style={{ animationDelay: `${i * 0.15}s` }}
                />
              ))}
            </div>
            <span>Analyzing…</span>
          </div>
        )}

        {/* Error */}
        {error && (
          <div className="text-xs text-red-400 bg-red-950/30 border border-red-900/50 rounded p-3 mb-3 leading-relaxed">
            {error}
          </div>
        )}

        {/* Prompt — no analysis triggered yet */}
        {!streaming && !error && insights.length === 0 && !output && (
          <div className="flex flex-col items-center justify-center h-full gap-3 text-center px-2">
            <span className="text-[#7a7adc] text-2xl leading-none">✦</span>
            <p className="text-xs text-text-muted leading-relaxed">
              Click Analyze to run AI analysis
            </p>
            {onAnalyze && (
              <button
                onClick={onAnalyze}
                className="h-6 px-3 rounded text-xxs font-mono bg-[#7a7adc]/20 text-[#7a7adc] border border-[#7a7adc]/40 hover:bg-[#7a7adc]/30 transition-colors"
              >
                ✦ Analyze
              </button>
            )}
          </div>
        )}

        {/* Insight cards */}
        {insights.map((insight, i) => (
          <InsightCard key={i} insight={insight} />
        ))}
      </div>
    </div>
  )
}
