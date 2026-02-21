// Individual pod row — Phase 1 Step 7
import type { PodSummary } from '@/types/kubernetes'

interface PodRowProps {
  pod: PodSummary
  isSelected: boolean
  onSelect: () => void
}

export function PodRow({ pod, isSelected, onSelect }: PodRowProps) {
  return (
    <tr
      className={`h-row cursor-pointer border-b border-border ${isSelected ? 'bg-accent/10' : 'hover:bg-surface'}`}
      onClick={onSelect}
    >
      <td className="px-3 text-xs">{pod.name}</td>
    </tr>
  )
}
