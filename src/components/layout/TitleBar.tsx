import { useState, useRef, useEffect } from 'react'
import { ChevronDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useClusterStore } from '@/store/clusterStore'
import { switchClusterContext } from '@/hooks/useCluster'
import type { ClusterHealth, KubeContext } from '@/types/kubernetes'
import { WindowControls } from './WindowControls'

// ── Health dot ────────────────────────────────────────────────────────────────

const healthDotClass: Record<ClusterHealth, string> = {
  healthy: 'bg-success',
  slow: 'bg-warning',
  unreachable: 'bg-error',
  unknown: 'bg-text-muted',
}

const healthTitle: Record<ClusterHealth, string> = {
  healthy: 'Reachable',
  slow: 'Slow response',
  unreachable: 'Unreachable',
  unknown: 'Checking…',
}

function HealthDot({ health }: { health: ClusterHealth }) {
  return (
    <span
      title={healthTitle[health]}
      className={cn(
        'inline-block w-1.5 h-1.5 rounded-full shrink-0',
        healthDotClass[health],
        health === 'healthy' && 'shadow-[0_0_4px_currentColor]',
      )}
    />
  )
}

// ── Cluster dropdown ──────────────────────────────────────────────────────────

interface ClusterDropdownProps {
  contexts: KubeContext[]
  activeContext: KubeContext | null
  healthMap: Record<string, ClusterHealth>
  onSelect: (ctx: KubeContext) => void
}

function ClusterDropdown({ contexts, activeContext, healthMap, onSelect }: ClusterDropdownProps) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    function handleOutside(e: MouseEvent) {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    if (open) document.addEventListener('mousedown', handleOutside)
    return () => document.removeEventListener('mousedown', handleOutside)
  }, [open])

  const displayName = activeContext
    ? (activeContext.name.length > 22 ? activeContext.name.slice(0, 20) + '…' : activeContext.name)
    : 'Select cluster'

  return (
    <div className="relative mx-2" ref={ref}>
      <button
        onClick={() => setOpen((o) => !o)}
        className={cn(
          'flex items-center justify-between gap-1 h-6 px-2 rounded',
          'bg-surface border border-border text-xxs font-mono text-text-primary',
          'hover:border-text-muted/40 focus:outline-none transition-colors',
        )}
        style={{ width: 200 }}
      >
        <div className="flex items-center gap-1.5 min-w-0 overflow-hidden">
          {activeContext && (
            <HealthDot health={healthMap[activeContext.name] ?? 'unknown'} />
          )}
          <span className="truncate">{displayName}</span>
        </div>
        <ChevronDown size={10} className="shrink-0 text-text-muted" />
      </button>

      {open && (
        <div
          className="absolute top-full left-0 mt-1 z-50 rounded border border-border bg-surface shadow-lg"
          style={{ width: 200 }}
        >
          <div className="max-h-64 overflow-y-auto py-1">
            {contexts.length === 0 ? (
              <div className="px-2 py-1.5 text-xxs font-mono text-text-muted/60">
                No clusters found
              </div>
            ) : (
              contexts.map((ctx) => (
                <button
                  key={ctx.name}
                  onClick={() => { onSelect(ctx); setOpen(false) }}
                  className={cn(
                    'w-full text-left flex items-center gap-1.5 px-2 py-1.5',
                    'text-xxs font-mono truncate transition-colors hover:bg-accent/10',
                    activeContext?.name === ctx.name ? 'text-accent' : 'text-text-muted',
                  )}
                  title={ctx.name}
                >
                  <HealthDot health={healthMap[ctx.name] ?? 'unknown'} />
                  <span className="truncate">{ctx.name}</span>
                </button>
              ))
            )}
          </div>
        </div>
      )}
    </div>
  )
}

// ── App logo ──────────────────────────────────────────────────────────────────

function AppLogo() {
  return (
    <div className="flex items-center gap-2 px-3 shrink-0 select-none">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden>
        <path
          d="M8 1.5L14 5v6L8 14.5 2 11V5L8 1.5z"
          stroke="#4a90d9"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
        <circle cx="8" cy="8" r="1.5" fill="#4a90d9" />
        <path d="M8 3.5v2M8 10.5v2M4.5 5.75l1.75 1M9.75 9.25l1.75 1M4.5 10.25l1.75-1M9.75 6.75l1.75-1"
          stroke="#4a90d9" strokeWidth="1" strokeLinecap="round" />
      </svg>
      <span className="text-xs font-semibold tracking-widest text-text-muted uppercase select-none">
        cluster-ops
      </span>
    </div>
  )
}

// ── TitleBar ──────────────────────────────────────────────────────────────────

export function TitleBar() {
  const { availableContexts, activeContext, healthMap, setActiveContext, setHealth } =
    useClusterStore()

  function handleSwitch(ctx: KubeContext) {
    switchClusterContext(ctx, setActiveContext, setHealth).catch(console.error)
  }

  return (
    <div
      data-tauri-drag-region
      className="flex items-center h-10 bg-surface border-b border-border select-none shrink-0"
    >
      {/* Left: logo + cluster dropdown */}
      <AppLogo />
      <div className="w-px h-5 bg-border shrink-0" />
      <ClusterDropdown
        contexts={availableContexts}
        activeContext={activeContext}
        healthMap={healthMap}
        onSelect={handleSwitch}
      />

      {/* Center: drag region spacer */}
      <div className="flex-1 min-w-0 self-stretch" data-tauri-drag-region />

      {/* Right: active cluster name */}
      {activeContext && (
        <>
          <div className="flex items-center gap-2 px-3 shrink-0">
            <HealthDot health={healthMap[activeContext.name] ?? 'unknown'} />
            <span className="text-xs font-bold font-mono text-white tracking-wide">
              {activeContext.name}
            </span>
          </div>
          <div className="w-px h-5 bg-border shrink-0" />
        </>
      )}

      {/* Window controls */}
      <WindowControls />
    </div>
  )
}
