// kubectl command bar — Phase 1 Step 13
// Features: history (↑/↓), tab completion with popup

import { useRef, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

import { cn } from '@/lib/utils'
import { useUIStore } from '@/store/uiStore'
import { useClusterStore } from '@/store/clusterStore'
import { useNamespaceStore } from '@/store/namespaceStore'

// ── Completion data ───────────────────────────────────────────────────────────

const SUBCOMMANDS = [
  'get', 'describe', 'logs', 'exec', 'apply', 'delete',
  'rollout', 'scale', 'top', 'port-forward', 'cp',
  'label', 'annotate', 'patch',
]

const RESOURCE_TYPES = [
  'pods', 'deployments', 'services', 'configmaps', 'secrets',
  'ingresses', 'namespaces', 'nodes', 'persistentvolumeclaims',
]

function getCompletions(input: string, namespaces: string[]): string[] {
  const t = input.trimStart()
  // "-n " or "--namespace " → all namespaces
  if (/\s-n\s+$/i.test(t) || /\s--namespace\s+$/i.test(t)) return namespaces
  // "-n partial" → filtered namespaces
  const nsMatch = t.match(/\s-n\s+(\S+)$/i) ?? t.match(/\s--namespace\s+(\S+)$/i)
  if (nsMatch) {
    return namespaces.filter((ns) => ns.startsWith(nsMatch[1]) && ns !== nsMatch[1])
  }
  // "kubectl get " → resource types
  if (/^kubectl\s+get\s+$/i.test(t)) return RESOURCE_TYPES
  // "kubectl " (just the word) → subcommands
  if (/^kubectl\s+$/i.test(t)) return SUBCOMMANDS
  // "kubectl ge…" (partial subcommand) → filtered subcommands
  const m = t.match(/^kubectl\s+(\S+)$/i)
  if (m) {
    const partial = m[1].toLowerCase()
    return SUBCOMMANDS.filter((s) => s.startsWith(partial) && s !== partial)
  }
  return []
}

// ── Component ─────────────────────────────────────────────────────────────────

export function CommandBar() {
  const {
    openOutputPanel,
    incrementCommandKey,
    addToCommandHistory,
    commandHistory,
  } = useUIStore()
  const activeContext = useClusterStore((s) => s.activeContext)
  const namespaces    = useNamespaceStore((s) => s.availableNamespaces)

  const [input, setInput]                   = useState('')
  const [completions, setCompletions]       = useState<string[]>([])
  const [completionIndex, setCompletionIndex] = useState(0)
  const [showCompletions, setShowCompletions] = useState(false)
  const [historyIndex, setHistoryIndex]     = useState(-1)  // -1 = live input

  const inputRef      = useRef<HTMLInputElement>(null)
  const savedInputRef = useRef('')   // preserves live input while browsing history
  const baseInputRef  = useRef('')   // input before Tab was first pressed (for cycling)

  // ── Completion helpers ──────────────────────────────────────────────────────

  const applyCompletion = (completion: string, base: string) => {
    const t = base.trimStart()
    if (/^kubectl\s+get\s+$/i.test(t)) {
      setInput(`kubectl get ${completion} `)
    } else if (/^kubectl\s+$/i.test(t)) {
      setInput(`kubectl ${completion} `)
    } else {
      // partial subcommand — replace the last non-whitespace run
      setInput(base.replace(/\S+$/, completion) + ' ')
    }
  }

  const hideCompletions = () => {
    setShowCompletions(false)
    setCompletions([])
    setCompletionIndex(0)
  }

  // ── Key handler ─────────────────────────────────────────────────────────────

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {

    // ── Enter: run command ──────────────────────────────────────────────────
    if (e.key === 'Enter') {
      e.preventDefault()
      const cmd = input.trim()
      if (!cmd) return

      hideCompletions()
      addToCommandHistory(cmd)
      setHistoryIndex(-1)
      savedInputRef.current = ''
      setInput('')

      openOutputPanel('command')
      incrementCommandKey()

      invoke('run_kubectl', {
        command: cmd,
        sourceFile:  activeContext?.sourceFile  ?? '',
        contextName: activeContext?.contextName ?? '',
      }).catch(console.error)
      return
    }

    // ── Arrow Up: go back in history ────────────────────────────────────────
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      if (commandHistory.length === 0) return
      if (historyIndex === -1) savedInputRef.current = input
      const next = Math.min(historyIndex + 1, commandHistory.length - 1)
      setHistoryIndex(next)
      setInput(commandHistory[commandHistory.length - 1 - next])
      hideCompletions()
      return
    }

    // ── Arrow Down: go forward in history ───────────────────────────────────
    if (e.key === 'ArrowDown') {
      e.preventDefault()
      if (historyIndex <= 0) {
        setHistoryIndex(-1)
        setInput(savedInputRef.current)
        hideCompletions()
        return
      }
      const next = historyIndex - 1
      setHistoryIndex(next)
      setInput(commandHistory[commandHistory.length - 1 - next])
      return
    }

    // ── Tab: complete ───────────────────────────────────────────────────────
    if (e.key === 'Tab') {
      e.preventDefault()

      if (!showCompletions) {
        // First Tab press — compute completions from current input
        const computed = getCompletions(input, namespaces)
        if (computed.length === 0) return
        baseInputRef.current = input
        setCompletions(computed)
        setCompletionIndex(0)
        setShowCompletions(true)
        applyCompletion(computed[0], input)
      } else {
        // Subsequent Tab presses — cycle through completions
        const next = (completionIndex + 1) % completions.length
        setCompletionIndex(next)
        applyCompletion(completions[next], baseInputRef.current)
      }
      return
    }

    // ── Escape: dismiss completions ─────────────────────────────────────────
    if (e.key === 'Escape') {
      e.preventDefault()
      hideCompletions()
      return
    }
  }

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setInput(e.target.value)
    setHistoryIndex(-1)
    hideCompletions()
  }

  // ── Render ──────────────────────────────────────────────────────────────────

  return (
    <div
      className="relative shrink-0 border-y border-border bg-surface"
      style={{ minHeight: '32px', display: 'block' }}
    >

      {/* Completions popup */}
      {showCompletions && completions.length > 0 && (
        <div className="absolute bottom-full left-0 mb-px bg-surface border border-border rounded shadow-lg z-50 py-1 min-w-[160px]">
          {completions.map((c, i) => (
            <button
              key={c}
              className={cn(
                'block w-full text-left px-3 py-1 text-xxs font-mono transition-colors',
                i === completionIndex
                  ? 'bg-accent/20 text-accent'
                  : 'text-text-muted hover:bg-white/5 hover:text-text-primary',
              )}
              onMouseDown={(e) => {
                e.preventDefault() // keep input focused
                applyCompletion(c, baseInputRef.current || input)
                hideCompletions()
                inputRef.current?.focus()
              }}
            >
              {c}
            </button>
          ))}
        </div>
      )}

      {/* Input row */}
      <div className="flex items-center h-8 px-3 gap-2">
        <span className="text-accent text-xs font-mono shrink-0 select-none">kubectl ▶</span>
        <input
          ref={inputRef}
          value={input}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          className="flex-1 bg-transparent text-xs font-mono text-text-primary outline-none placeholder:text-text-muted/50"
          placeholder="type a command…"
          spellCheck={false}
          autoComplete="off"
          autoCapitalize="off"
        />
      </div>
    </div>
  )
}
