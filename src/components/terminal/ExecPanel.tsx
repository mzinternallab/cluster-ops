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
  const { selectedPod, execSessionKey } = useUIStore()
  const activeContext   = useClusterStore((s) => s.activeContext)
  const containerRef    = useRef<HTMLDivElement>(null)

  useEffect(() => {
    console.log('[exec] useEffect fired', {
      selectedPod,
      activeContext,
      execSessionKey,
    })
    const el = containerRef.current
    if (!el || !selectedPod || !activeContext) return

    // ── Terminal ─────────────────────────────────────────────────────────────

    const term = new Terminal({
      theme:        TERM_THEME,
      fontFamily:   'JetBrains Mono, Cascadia Code, Consolas, monospace',
      fontSize:     12,
      lineHeight:   1.4,
      cursorBlink:  true,
      disableStdin: false,   // keyboard input is forwarded to the PTY
      scrollback:   5_000,
    })

    const fit = new FitAddon()
    term.loadAddon(fit)
    term.open(el)
    fit.fit()
    term.focus()

    const ro = new ResizeObserver(() => fit.fit())
    ro.observe(el)

    // ── Event wiring ─────────────────────────────────────────────────────────

    let active = true
    const unlisten: (() => void)[] = []

    // Register listeners BEFORE invoking to guarantee no bytes are missed.
    // PTY handles echoing, prompts, and ANSI sequences — write raw output directly.
    Promise.all([
      listen<string>('exec-output', (e) => { if (active) term.write(e.payload) }),
      listen<null>  ('exec-done',   ()  => {
        if (!active) return
        term.writeln('\r\n\x1b[2m[session ended]\x1b[0m')
        // Clear terminal after 2 s so the next session starts with a blank slate.
        setTimeout(() => { if (active) term.clear() }, 2000)
      }),
    ]).then((fns) => {
      if (!active) { fns.forEach((f) => f()); return }
      unlisten.push(...fns)

      console.log('[exec] calling invoke exec_into_pod')
      invoke('exec_into_pod', {
        name:        selectedPod.name,
        namespace:   selectedPod.namespace,
        sourceFile:  activeContext.sourceFile,
        contextName: activeContext.contextName,
      }).catch((err: unknown) => {
        console.error('[exec] invoke failed:', err)
        if (active) term.writeln(`\r\n\x1b[31mError: ${String(err)}\x1b[0m`)
      })
    })

    // Forward raw xterm.js keystrokes to the PTY master — no buffering needed.
    const onData = term.onData((data) => {
      invoke('send_exec_input', { input: data }).catch(() => {})
    })

    // ── Cleanup ───────────────────────────────────────────────────────────────

    return () => {
      console.log('[exec] cleanup fired')
      active = false
      onData.dispose()
      ro.disconnect()
      unlisten.forEach((f) => f())
      term.dispose()
    }
  // Re-run when the target pod / cluster changes.
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [selectedPod?.name, selectedPod?.namespace, activeContext?.sourceFile, activeContext?.contextName, execSessionKey])

  return (
    <div
      ref={containerRef}
      className="flex-1 overflow-hidden"
      style={{ padding: '4px 8px' }}
    />
  )
}
