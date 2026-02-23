import { useState } from 'react'
import { useQueryClient } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { FileText, Search, Trash2 } from 'lucide-react'
import { cn } from '@/lib/utils'
import type { PodSummary } from '@/types/kubernetes'
import { StatusBadge } from './StatusBadge'

interface PodRowProps {
  pod: PodSummary
  isSelected: boolean
  onSelect: () => void
  onDescribe: () => void
  onLogs: () => void
  onExec: () => void
}

export function PodRow({ pod, isSelected, onSelect, onDescribe, onLogs, onExec }: PodRowProps) {
  const queryClient = useQueryClient()
  const [confirmDelete, setConfirmDelete] = useState(false)

  async function handleDelete() {
    try {
      await invoke('delete_pod', { name: pod.name, namespace: pod.namespace })
      await queryClient.invalidateQueries({ queryKey: ['pods'] })
    } catch (err) {
      console.error('delete_pod failed:', err)
    }
  }

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
      <td className="px-2 py-0">
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

      {/* COMMANDS cell */}
      <td className="px-2 py-0">
        <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-75">

          {/* Exec button */}
          <button
            title="Opens in terminal"
            onClick={(e) => { e.stopPropagation(); onExec() }}
            className="flex items-center h-5 px-1.5 rounded text-[10px] font-mono border transition-colors"
            style={{ background: '#1a1a2e', borderColor: '#2a2a4a', color: '#7a7adc' }}
          >
            &gt;_
          </button>

          {/* Delete button */}
          <button
            title="Delete pod"
            onClick={(e) => { e.stopPropagation(); setConfirmDelete(true) }}
            className="flex items-center h-5 px-1.5 rounded border transition-colors"
            style={{ background: '#2a1a1a', borderColor: '#4a2a2a', color: '#ef4444' }}
          >
            <Trash2 size={10} />
          </button>

          {/* Describe button */}
          <button
            title="Describe"
            onClick={(e) => { e.stopPropagation(); onDescribe() }}
            className="p-1 rounded hover:bg-white/10 text-text-muted hover:text-text-primary"
          >
            <Search size={12} />
          </button>

          {/* Logs button */}
          <button
            title="Logs"
            onClick={(e) => { e.stopPropagation(); onLogs() }}
            className="p-1 rounded hover:bg-white/10 text-text-muted hover:text-text-primary"
          >
            <FileText size={12} />
          </button>
        </div>

        {/* Delete confirmation modal (fixed overlay — breaks out of table stacking) */}
        {confirmDelete && (
          <div
            className="fixed inset-0 z-[100] flex items-center justify-center bg-black/60"
            onClick={(e) => { e.stopPropagation(); setConfirmDelete(false) }}
          >
            <div
              className="bg-surface border border-border rounded-lg p-5 w-[360px] shadow-2xl"
              onClick={(e) => e.stopPropagation()}
            >
              <h3 className="text-sm font-semibold font-mono text-text-primary mb-2">
                Delete Pod
              </h3>
              <p className="text-xs font-mono text-text-muted mb-5">
                Are you sure you want to delete{' '}
                <span className="text-text-primary">{pod.name}</span>?
              </p>
              <div className="flex justify-end gap-2">
                <button
                  onClick={() => setConfirmDelete(false)}
                  className="h-7 px-3 rounded text-xs font-mono border border-border text-text-muted hover:text-text-primary hover:border-text-muted/40 transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={() => { setConfirmDelete(false); void handleDelete() }}
                  className="h-7 px-3 rounded text-xs font-mono border transition-colors"
                  style={{ background: '#2a1a1a', borderColor: '#4a2a2a', color: '#ef4444' }}
                >
                  Delete
                </button>
              </div>
            </div>
          </div>
        )}
      </td>
    </tr>
  )
}
