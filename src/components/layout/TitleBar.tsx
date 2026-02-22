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

interface HealthDotProps {
  health: ClusterHealth
}

function HealthDot({ health }: HealthDotProps) {
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

// ── Cluster tab ───────────────────────────────────────────────────────────────

interface ClusterTabProps {
  ctx: KubeContext
  health: ClusterHealth
  isActive: boolean
  onClick: () => void
}

function ClusterTab({ ctx, health, isActive, onClick }: ClusterTabProps) {
  // Truncate long context names
  const displayName = ctx.name.length > 22 ? ctx.name.slice(0, 20) + '…' : ctx.name

  return (
    <button
      onClick={onClick}
      title={ctx.name}
      className={cn(
        'flex items-center gap-1.5 h-10 px-3 text-xs font-mono transition-colors duration-100',
        'border-b-2 focus-visible:outline-none shrink-0',
        isActive
          ? 'text-text-primary border-accent bg-white/4'
          : 'text-text-muted border-transparent hover:text-text-primary hover:bg-white/4',
      )}
    >
      <HealthDot health={health} />
      {displayName}
    </button>
  )
}

// ── App logo ──────────────────────────────────────────────────────────────────

function AppLogo() {
  return (
    <div className="flex items-center gap-2 px-3 shrink-0 select-none">
      {/* Hexagon / k8s-inspired icon */}
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden>
        <path
          d="M8 1.5L14 5v6L8 14.5 2 11V5L8 1.5z"
          stroke="#4a90d9"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
        <circle cx="8" cy="8" r="1.5" fill="#4a90d9" />
        {/* Spokes */}
        <path d="M8 3.5v2M8 10.5v2M4.5 5.75l1.75 1M9.75 9.25l1.75 1M4.5 10.25l1.75-1M9.75 6.75l1.75-1"
          stroke="#4a90d9" strokeWidth="1" strokeLinecap="round" />
      </svg>
      <span className="text-xs font-semibold tracking-widest text-text-muted uppercase select-none">
        cluster-ops
      </span>
    </div>
  )
}

// ── No clusters empty state ───────────────────────────────────────────────────

function NoClusters() {
  return (
    <div className="flex items-center px-3 h-10">
      <span className="text-xxs text-text-muted italic">
        No clusters — open kubeconfig to connect
      </span>
    </div>
  )
}

// ── TitleBar ──────────────────────────────────────────────────────────────────

export function TitleBar() {
  const { availableContexts, activeContext, healthMap, setActiveContext, setHealth } =
    useClusterStore()

  function handleSwitch(ctx: KubeContext) {
    // Fire-and-forget — persists to kubeconfig and re-pings health
    switchClusterContext(ctx, setActiveContext, setHealth).catch(console.error)
  }

  return (
    <div
      data-tauri-drag-region
      className="flex items-center h-10 bg-surface border-b border-border select-none shrink-0"
    >
      {/* Left: branding */}
      <AppLogo />

      {/* Separator */}
      <div className="w-px h-5 bg-border shrink-0" />

      {/* Center: cluster tabs — scrollable if many clusters */}
      <div className="flex flex-1 items-center overflow-x-auto overflow-y-hidden scrollbar-none min-w-0">
        {availableContexts.length === 0 ? (
          <NoClusters />
        ) : (
          availableContexts.map((ctx) => (
            <ClusterTab
              key={ctx.name}
              ctx={ctx}
              health={healthMap[ctx.name] ?? 'unknown'}
              isActive={activeContext?.name === ctx.name}
              onClick={() => handleSwitch(ctx)}
            />
          ))
        )}
      </div>

      {/* Right: window controls */}
      <WindowControls />
    </div>
  )
}
