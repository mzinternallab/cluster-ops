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

// ── chevron ───────────────────────────────────────────────────────────────────

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

// ── checkmark ─────────────────────────────────────────────────────────────────

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

  // Close on outside click
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

  // Close on Escape
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

  const activeHealth: ClusterHealth =
    activeContext ? (healthMap[activeContext.name] ?? 'unknown') : 'unknown'

  const activeLabel = activeContext
    ? activeContext.name.length > 24
      ? activeContext.name.slice(0, 22) + '…'
      : activeContext.name
    : 'No cluster'

  return (
    <div className="flex items-center gap-2 shrink-0">
      {/* Label */}
      <span className="text-[10px] font-mono font-semibold tracking-[0.15em] text-text-muted uppercase select-none">
        Cluster:
      </span>

      {/* Button + dropdown wrapper */}
      <div ref={containerRef} className="relative">
        {/* Trigger */}
        <button
          onClick={() => setOpen((o) => !o)}
          title={activeContext?.name}
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

        {/* Dropdown list */}
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
                const health: ClusterHealth = healthMap[ctx.name] ?? 'unknown'
                const isActive = activeContext?.name === ctx.name
                const label =
                  ctx.name.length > 26 ? ctx.name.slice(0, 24) + '…' : ctx.name
                return (
                  <button
                    key={ctx.name}
                    onClick={() => handleSelect(ctx)}
                    title={ctx.name}
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
