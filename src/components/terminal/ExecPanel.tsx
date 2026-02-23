import { useEffect, useRef } from 'react'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import '@xterm/xterm/css/xterm.css'

import { useClusterStore } from '@/store/clusterStore'
import { useUIStore } from '@/store/uiStore'

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

// ── ExecPanel ─────────────────────────────────────────────────────────────────

export function ExecPanel() {
  const { selectedPod } = useUIStore()
  const activeContext = useClusterStore((s) => s.activeContext)

  const containerRef = useRef<HTMLDivElement>(null)
  const termRef      = useRef<Terminal | null>(null)
  const fitRef       = useRef<FitAddon | null>(null)

  useEffect(() => {
    const el = containerRef.current
    if (!el || !selectedPod || !activeContext) return

    // ── 1. Build terminal ────────────────────────────────────────────────────

    const term = new Terminal({
      theme:       TERM_THEME,
      fontFamily:  'JetBrains Mono, Cascadia Code, Consolas, monospace',
      fontSize:    12,
      lineHeight:  1.4,
      cursorBlink: true,
      disableStdin: false,   // allow keyboard input
      scrollback:  5_000,
    })

    const fit = new FitAddon()
    term.loadAddon(fit)
    term.open(el)
    fit.fit()
    term.focus()

    termRef.current = term
    fitRef.current  = fit

    // ── 2. Sync terminal size on container resize ────────────────────────────

    const ro = new ResizeObserver(() => {
      fitRef.current?.fit()
      const dims = fitRef.current?.proposeDimensions()
      if (!dims) return
      invoke('resize_pty', { cols: dims.cols, rows: dims.rows }).catch(() => {})
    })
    ro.observe(el)

    // ── 3. Subscribe to PTY output events ───────────────────────────────────

    let active = true
    const unlisten: (() => void)[] = []

    term.write(`\x1b[2mConnecting to ${selectedPod.namespace}/${selectedPod.name}...\x1b[0m\r\n`)

    Promise.all([
      listen<string>('pty-output', (e) => {
        if (active) term.write(e.payload)
      }),
      listen<null>('pty-done', () => {
        if (active) term.write('\r\n\x1b[2m[session ended]\x1b[0m\r\n')
      }),
    ]).then((fns) => {
      if (!active) { fns.forEach((f) => f()); return }
      unlisten.push(...fns)

      // ── 4. Start PTY session ─────────────────────────────────────────────

      const dims = fit.proposeDimensions() ?? { cols: 80, rows: 24 }
      invoke('start_pty_exec', {
        name:        selectedPod.name,
        namespace:   selectedPod.namespace,
        sourceFile:  activeContext.sourceFile,
        contextName: activeContext.contextName,
        cols:        dims.cols,
        rows:        dims.rows,
      }).catch((err: unknown) => {
        if (active) {
          term.write(`\r\n\x1b[31mError: ${String(err)}\x1b[0m\r\n`)
        }
      })
    })

    // ── 5. Forward keystrokes → PTY ─────────────────────────────────────────

    const onData = term.onData((data) => {
      invoke('send_exec_input', { data }).catch(() => {})
    })

    // ── 6. Cleanup ───────────────────────────────────────────────────────────

    return () => {
      active = false
      onData.dispose()
      ro.disconnect()
      unlisten.forEach((f) => f())
      invoke('stop_pty_exec').catch(() => {})
      term.dispose()
      termRef.current = null
      fitRef.current  = null
    }
  // Re-run only when the target pod or cluster changes.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedPod?.name, selectedPod?.namespace, activeContext?.sourceFile, activeContext?.contextName])

  return (
    <div
      ref={containerRef}
      className="flex-1 overflow-hidden"
      style={{ padding: '4px 8px' }}
    />
  )
}
