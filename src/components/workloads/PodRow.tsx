import { FileText, Search } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { PodSummary } from '@/types/kubernetes'
import { StatusBadge } from './StatusBadge'

interface PodRowProps {
  pod: PodSummary
  isSelected: boolean
  onSelect: () => void
  onDescribe: () => void
  onLogs: () => void
}

export function PodRow({ pod, isSelected, onSelect, onDescribe, onLogs }: PodRowProps) {
  return (
    <tr
      className={cn(
        'h-row cursor-pointer border-b border-border group transition-colors duration-75',
        isSelected ? 'bg-accent/10' : 'hover:bg-surface',
      )}
      onClick={onSelect}
    >
      <td className="px-3 py-0 text-xs font-mono text-text-primary truncate max-w-[180px]">
        {pod.name}
      </td>
      <td className="px-3 py-0 text-xs font-mono text-text-muted truncate max-w-[120px]">
        {pod.namespace}
      </td>
      <td className="px-3 py-0">
        <StatusBadge status={pod.status} />
      </td>
      <td className="px-3 py-0 text-xs font-mono text-text-muted">{pod.ready}</td>
      <td
        className={cn(
          'px-3 py-0 text-xs font-mono',
          pod.restarts > 5 ? 'text-error font-bold' : 'text-text-muted',
        )}
      >
        {pod.restarts}
      </td>
      <td className="px-3 py-0 text-xs font-mono text-text-muted">{pod.age}</td>
      <td className="px-3 py-0 text-xs font-mono text-text-muted">{pod.cpu}</td>
      <td className="px-3 py-0 text-xs font-mono text-text-muted">{pod.memory}</td>
      <td className="px-3 py-0 text-xs font-mono text-text-muted truncate max-w-[140px]">
        {pod.node}
      </td>
      {/* Action buttons — fade in on row hover */}
      <td className="px-2 py-0">
        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-75">
          <button
            title="Describe"
            onClick={(e) => { e.stopPropagation(); onDescribe() }}
            className="p-1 rounded hover:bg-white/10 text-text-muted hover:text-text-primary"
          >
            <Search size={12} />
          </button>
          <button
            title="Logs"
            onClick={(e) => { e.stopPropagation(); onLogs() }}
            className="p-1 rounded hover:bg-white/10 text-text-muted hover:text-text-primary"
          >
            <FileText size={12} />
          </button>
        </div>
      </td>
    </tr>
  )
}
