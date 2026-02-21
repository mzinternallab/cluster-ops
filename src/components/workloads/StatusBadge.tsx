// Status pill component — Phase 1 Step 7
import { cn } from '@/lib/utils'
import type { PodStatus } from '@/types/kubernetes'

interface StatusBadgeProps {
  status: PodStatus | string
}

const statusColors: Record<string, string> = {
  Running: 'bg-success/20 text-success',
  Pending: 'bg-warning/20 text-warning',
  Terminating: 'bg-orange-500/20 text-orange-400',
  CrashLoopBackOff: 'bg-error/20 text-error',
  OOMKilled: 'bg-error/20 text-error',
  Error: 'bg-error/20 text-error',
  Completed: 'bg-text-muted/20 text-text-muted',
}

export function StatusBadge({ status }: StatusBadgeProps) {
  const color = statusColors[status] ?? 'bg-text-muted/20 text-text-muted'
  return (
    <span className={cn('inline-flex items-center px-1.5 py-0.5 rounded text-xxs font-mono', color)}>
      {status}
    </span>
  )
}
