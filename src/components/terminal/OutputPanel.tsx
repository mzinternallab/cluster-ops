import { useEffect, useRef, useState } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { X } from 'lucide-react'
import '@xterm/xterm/css/xterm.css'

import { cn } from '@/lib/utils'
import { useUIStore } from '@/store/uiStore'
import { useClusterStore } from '@/store/clusterStore'
import { ExecPanel } from './ExecPanel'

// ── ANSI helpers ──────────────────────────────────────────────────────────────

const RED    = '\x1b[31m'
const YELLOW = '\x1b[33m'
const RESET  = '\x1b[0m'

const ERROR_RE = /error|fatal|oomkill|crashloop/i
const WARN_RE  = /warning|backoff/i

function highlightLine(line: string): string {
  if (ERROR_RE.test(line)) return `${RED}${line}${RESET}`
  if (WARN_RE.test(line))  return `${YELLOW}${line}${RESET}`
  return line
}

// ── Tail options ──────────────────────────────────────────────────────────────

const TAIL_OPTIONS: { label: string; value: number | null }[] = [
  { label: '50',  value: 50   },
  { label: '100', value: 100  },
  { label: '200', value: 200  },
  { label: '500', value: 500  },
  { label: 'all', value: null },
]

// ── Terminal theme (SPEC.md §7) ───────────────────────────────────────────────

const TERM_THEME = {
  background:          '#0a0e1a',
  foreground:          '#c9d1e9',
  cursor:              '#4a90d9',
  selectionBackground: 'rgba(74, 144, 217, 0.25)',
  black:               '#0a0e1a',
  brightBlack:         '#4a6a8a',
  red:                 '#ef4444',
  brightRed:           '#ef4444',
  green:               '#22c55e',
  brightGreen:         '#22c55e',
  yellow:              '#f59e0b',
  brightYellow:        '#f59e0b',
  blue:                '#4a90d9',
  brightBlue:          '#4a90d9',
  magenta:             '#7a7adc',
  brightMagenta:       '#7a7adc',
  cyan:                '#22d3ee',
  brightCyan:          '#22d3ee',
  white:               '#c9d1e9',
  brightWhite:         '#ffffff',
}

// ── Component ─────────────────────────────────────────────────────────────────

export function OutputPanel() {
  const { selectedPod, outputPanelMode, closeOutputPanel } = useUIStore()
  const activeContext = useClusterStore((s) => s.activeContext)

  const containerRef = useRef<HTMLDivElement>(null)
  const termRef      = useRef<Terminal | null>(null)
  const fitRef       = useRef<FitAddon | null>(null)
  const unlistensRef = useRef<(() => void)[]>([])

  const [tailLines,   setTailLines]   = useState<number | null>(200)
  const [follow,      setFollow]      = useState(true)
  const [isStreaming, setIsStreaming] = useState(false)

  // ── Terminal init (once on mount) ────────────────────────────────────────

  useEffect(() => {
    const el = containerRef.current
    if (!el) return

    const term = new Terminal({
      theme:       TERM_THEME,
      fontFamily:  'JetBrains Mono, Cascadia Code, Consolas, monospace',
      fontSize:    12,
      lineHeight:  1.4,
      cursorBlink: false,
      disableStdin: true,
      scrollback:  10_000,
    })

    const fit = new FitAddon()
    term.loadAddon(fit)
    term.open(el)
    fit.fit()

    termRef.current = term
    fitRef.current  = fit

    const ro = new ResizeObserver(() => fitRef.current?.fit())
    ro.observe(el)

    return () => {
      ro.disconnect()
      term.dispose()
      termRef.current = null
      fitRef.current  = null
    }
  }, [])

  // ── Data loading (describe or logs) ──────────────────────────────────────

  useEffect(() => {
    const term = termRef.current
    if (!term || !selectedPod || !outputPanelMode) return
    // exec mode is handled entirely by ExecPanel — nothing to do here.
    if (outputPanelMode === 'exec') return

    // Cancel any previous stream and remove listeners
    let active = true
    const stopListeners = () => {
      unlistensRef.current.forEach((fn) => fn())
      unlistensRef.current = []
    }
    stopListeners()

    term.clear()
    setIsStreaming(true)

    if (outputPanelMode === 'describe') {
      invoke<string>('describe_pod', {
        name:        selectedPod.name,
        namespace:   selectedPod.namespace,
        sourceFile:  activeContext?.sourceFile  ?? '',
        contextName: activeContext?.contextName ?? '',
      })
        .then((output) => {
          if (!active) return
          output.split(/\r?\n/).forEach((line) => term.writeln(highlightLine(line)))
          setIsStreaming(false)
        })
        .catch((err: unknown) => {
          if (!active) return
          term.writeln(`${RED}${String(err)}${RESET}`)
          setIsStreaming(false)
        })
    } else if (outputPanelMode === 'logs') {
      // Set up listeners BEFORE invoking so no events are missed
      Promise.all([
        listen<string>('pod-log-line',  (e) => { if (active) term.writeln(highlightLine(e.payload)) }),
        listen<string>('pod-log-error', (e) => { if (active) term.writeln(`${RED}${e.payload}${RESET}`) }),
        listen<null>  ('pod-log-done',  ()  => { if (active) setIsStreaming(false) }),
      ]).then((uls) => {
        if (!active) { uls.forEach((fn) => fn()); return }
        unlistensRef.current = uls

        invoke('get_pod_logs', {
          name:        selectedPod.name,
          namespace:   selectedPod.namespace,
          sourceFile:  activeContext?.sourceFile  ?? '',
          contextName: activeContext?.contextName ?? '',
          tail:        tailLines,
          follow,
        }).catch((err: unknown) => {
          if (!active) return
          term.writeln(`${RED}${String(err)}${RESET}`)
          setIsStreaming(false)
        })
      })
    }

    return () => {
      active = false
      stopListeners()
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedPod?.name, selectedPod?.namespace, outputPanelMode, tailLines, follow])

  // ── Render ────────────────────────────────────────────────────────────────

  const isLogs = outputPanelMode === 'logs'
  const isExec = outputPanelMode === 'exec'
  const title  = selectedPod
    ? `${selectedPod.namespace}/${selectedPod.name}`
    : ''

  return (
    <div className="flex flex-col h-full overflow-hidden">

      {/* Header */}
      <div className="flex items-center gap-2 px-3 h-8 bg-surface border-b border-border shrink-0">

        {/* Mode badge */}
        <span className={cn(
          'text-xxs font-mono px-1.5 py-0.5 rounded uppercase tracking-wider shrink-0',
          isLogs ? 'bg-accent/15 text-accent'
            : isExec ? 'bg-[#1a1a2e] text-[#7a7adc]'
            : 'bg-ai-purple/15 text-ai-purple',
        )}>
          {outputPanelMode ?? 'output'}
        </span>

        {/* Pod title */}
        <span className="text-xs font-mono text-text-muted truncate flex-1 min-w-0">
          {title}
        </span>

        {/* Streaming indicator */}
        {isStreaming && (
          <span className="text-xxs text-text-muted animate-pulse shrink-0">
            streaming…
          </span>
        )}

        {/* Logs-only controls */}
        {isLogs && (
          <>
            {/* Tail line selector */}
            <select
              value={tailLines ?? 'all'}
              onChange={(e) => {
                const v = e.target.value
                setTailLines(v === 'all' ? null : Number(v))
              }}
              className={cn(
                'h-5 px-1 bg-background border border-border rounded',
                'text-xxs font-mono text-text-muted',
                'focus:outline-none focus:border-accent shrink-0',
              )}
            >
              {TAIL_OPTIONS.map((o) => (
                <option key={o.label} value={o.value ?? 'all'}>
                  {o.label} lines
                </option>
              ))}
            </select>

            {/* Follow toggle */}
            <button
              onClick={() => setFollow((f) => !f)}
              className={cn(
                'h-5 px-2 rounded text-xxs font-mono border transition-colors shrink-0',
                follow
                  ? 'bg-accent/15 text-accent border-accent/40'
                  : 'text-text-muted border-border hover:border-text-muted/40 hover:text-text-primary',
              )}
            >
              follow
            </button>
          </>
        )}

        {/* Close button */}
        <button
          onClick={closeOutputPanel}
          className="p-0.5 rounded hover:bg-white/10 text-text-muted hover:text-text-primary shrink-0"
          title="Close panel"
        >
          <X size={14} />
        </button>
      </div>

      {/* Terminal area */}
      {isExec
        ? <ExecPanel />
        : <div ref={containerRef} className="flex-1 overflow-hidden" style={{ padding: '4px 8px' }} />
      }
    </div>
  )
}
