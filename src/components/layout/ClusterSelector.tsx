import { useState, useRef, useEffect } from 'react'
import { cn } from '@/lib/utils'
import { useClusterStore } from '@/store/clusterStore'
import { switchClusterContext } from '@/hooks/useCluster'
import type { ClusterHealth, KubeContext } from '@/types/kubernetes'

// ── health dot ────────────────────────────────────────────────────────────────

const dotClass: Record<ClusterHealth, string> = {
  healthy:     'bg-success shadow-[0_0_4px_#22c55e]',
  slow:        'bg-warning',
  unreachable: 'bg-error',
  unknown:     'bg-text-muted',
}

const dotTitle: Record<ClusterHealth, string> = {
  healthy:     'Reachable',
  slow:        'Slow response',
  unreachable: 'Unreachable',
  unknown:     'Checking…',
}

function Dot({ health }: { health: ClusterHealth }) {
  return (
    <span
      title={dotTitle[health]}
      className={cn('inline-block w-1.5 h-1.5 rounded-full shrink-0', dotClass[health])}
    />
  )
}

function Chevron({ open }: { open: boolean }) {
  return (
    <svg
      width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden
      className={cn(
        'shrink-0 text-text-muted transition-transform duration-150',
        open && 'rotate-180',
      )}
    >
      <path
        d="M2 3.5L5 6.5L8 3.5"
        stroke="currentColor" strokeWidth="1.3"
        strokeLinecap="round" strokeLinejoin="round"
      />
    </svg>
  )
}

function Check() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden
      className="shrink-0 text-accent"
    >
      <path
        d="M2 5L4.2 7.5L8 3"
        stroke="currentColor" strokeWidth="1.4"
        strokeLinecap="round" strokeLinejoin="round"
      />
    </svg>
  )
}

// ── ClusterSelector ──────────────────────────────────────────────────────────

export function ClusterSelector() {
  const { availableContexts, activeContext, healthMap, setActiveContext, setHealth } =
    useClusterStore()
  const [open, setOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!open) return
    function onMouseDown(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false)
      }
    }
    document.addEventListener('mousedown', onMouseDown)
    return () => document.removeEventListener('mousedown', onMouseDown)
  }, [open])

  useEffect(() => {
    if (!open) return
    function onKey(e: KeyboardEvent) {
      if (e.key === 'Escape') setOpen(false)
    }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [open])

  function handleSelect(ctx: KubeContext) {
    setOpen(false)
    switchClusterContext(ctx, setActiveContext, setHealth).catch(console.error)
  }

  // healthMap is keyed by displayName (unique per cluster, even when contextName is "local" for all).
  const activeHealth: ClusterHealth =
    activeContext ? (healthMap[activeContext.displayName] ?? 'unknown') : 'unknown'

  const activeLabel = activeContext
    ? activeContext.displayName.length > 24
      ? activeContext.displayName.slice(0, 22) + '…'
      : activeContext.displayName
    : 'No cluster'

  return (
    <div className="flex items-center gap-2 shrink-0">
      <span className="text-[10px] font-mono font-semibold tracking-[0.15em] text-text-muted uppercase select-none">
        Cluster:
      </span>

      <div ref={containerRef} className="relative">
        <button
          onClick={() => setOpen((o) => !o)}
          // Show contextName in tooltip so user can see the underlying k8s context
          title={activeContext ? `${activeContext.displayName} (context: ${activeContext.contextName})` : undefined}
          className={cn(
            'flex items-center gap-2 h-7 px-3 rounded',
            'w-[220px] font-mono text-xs',
            'border border-border bg-background',
            'hover:border-accent/60 transition-colors duration-100',
            'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-accent/60',
            open && 'border-accent/60 bg-white/[0.03]',
          )}
        >
          <Dot health={activeHealth} />
          <span className="flex-1 text-left font-bold text-white truncate leading-none">
            {activeLabel}
          </span>
          <Chevron open={open} />
        </button>

        {open && (
          <div
            className={cn(
              'absolute top-full left-0 mt-1 z-50',
              'w-[220px] rounded border border-border',
              'bg-surface shadow-xl shadow-black/60',
              'py-1 overflow-hidden',
            )}
          >
            {availableContexts.length === 0 ? (
              <div className="px-3 py-2 text-[11px] text-text-muted italic font-mono">
                No clusters found
              </div>
            ) : (
              availableContexts.map((ctx) => {
                // healthMap keyed by displayName — unique even when contextName is "local" for all
                const health: ClusterHealth = healthMap[ctx.displayName] ?? 'unknown'
                const isActive = activeContext?.displayName === ctx.displayName
                const label =
                  ctx.displayName.length > 26 ? ctx.displayName.slice(0, 24) + '…' : ctx.displayName
                return (
                  <button
                    // sourceFile is always unique — safe key even with duplicate contextNames
                    key={ctx.sourceFile}
                    onClick={() => handleSelect(ctx)}
                    title={`${ctx.displayName} (context: ${ctx.contextName})`}
                    className={cn(
                      'w-full flex items-center gap-2 px-3 py-1.5',
                      'text-xs font-mono text-left transition-colors duration-75',
                      'focus-visible:outline-none',
                      isActive
                        ? 'bg-accent/10 text-white'
                        : 'text-text-muted hover:bg-white/5 hover:text-text-primary',
                    )}
                  >
                    <Dot health={health} />
                    <span className={cn('flex-1 truncate', isActive && 'font-semibold text-white')}>
                      {label}
                    </span>
                    {isActive && <Check />}
                  </button>
                )
              })
            )}
          </div>
        )}
      </div>
    </div>
  )
}
