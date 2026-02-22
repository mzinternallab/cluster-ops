import { useState, useMemo } from 'react'
import { ChevronUp, ChevronDown, ChevronsUpDown } from 'lucide-react'
import { cn } from '@/lib/utils'
import { usePods } from '@/hooks/usePods'
import { useUIStore } from '@/store/uiStore'
import { useNamespaceStore } from '@/store/namespaceStore'
import { PodRow } from './PodRow'
import type { PodSummary } from '@/types/kubernetes'

// ── Sort types ────────────────────────────────────────────────────────────────

type SortKey = keyof Pick<PodSummary, 'name' | 'namespace' | 'status' | 'ready' | 'restarts' | 'age' | 'node'>
type SortDir = 'asc' | 'desc'

// ── Column definitions ────────────────────────────────────────────────────────

const COLUMNS: { key: SortKey | null; label: string; className: string }[] = [
  { key: 'name',      label: 'NAME',     className: 'min-w-[160px]' },
  { key: 'namespace', label: 'NAMESPACE',className: 'min-w-[110px]' },
  { key: 'status',    label: 'STATUS',   className: 'min-w-[130px]' },
  { key: 'ready',     label: 'READY',    className: 'w-[70px]'      },
  { key: 'restarts',  label: 'RESTARTS', className: 'w-[80px]'      },
  { key: 'age',       label: 'AGE',      className: 'w-[65px]'      },
  { key: null,        label: 'CPU',      className: 'w-[65px]'      },
  { key: null,        label: 'MEM',      className: 'w-[70px]'      },
  { key: 'node',      label: 'NODE',     className: 'min-w-[120px]' },
  { key: null,        label: '',         className: 'w-[60px]'      },
]

// ── Helpers ───────────────────────────────────────────────────────────────────

function compareValue(a: PodSummary, b: PodSummary, key: SortKey): number {
  if (key === 'restarts') return a.restarts - b.restarts
  return String(a[key]).localeCompare(String(b[key]))
}

interface SortIconProps { col: SortKey | null; sortKey: SortKey; sortDir: SortDir }

function SortIcon({ col, sortKey, sortDir }: SortIconProps) {
  if (col !== sortKey) return <ChevronsUpDown size={10} className="ml-0.5 text-text-muted/40" />
  return sortDir === 'asc'
    ? <ChevronUp   size={10} className="ml-0.5 text-accent" />
    : <ChevronDown size={10} className="ml-0.5 text-accent" />
}

// ── Component ─────────────────────────────────────────────────────────────────

export function PodTable() {
  const { data: pods, isLoading, error } = usePods()
  const { selectedPod, setSelectedPod, openOutputPanel } = useUIStore()
  // Read activeNamespace directly so the client-side filter is applied
  // immediately — including while keepPreviousData is serving the old list.
  const activeNamespace = useNamespaceStore((s) => s.activeNamespace)

  const [search,  setSearch]  = useState('')
  const [sortKey, setSortKey] = useState<SortKey>('name')
  const [sortDir, setSortDir] = useState<SortDir>('asc')

  function handleSort(key: SortKey | null) {
    if (!key) return
    if (key === sortKey) {
      setSortDir((d) => (d === 'asc' ? 'desc' : 'asc'))
    } else {
      setSortKey(key)
      setSortDir('asc')
    }
  }

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase()
    // Start with server-fetched list (or keepPreviousData placeholder)
    let list = pods ?? []
    // Client-side namespace filter: applied immediately on every render,
    // so the correct subset is visible even before the namespaced backend
    // query completes.
    if (activeNamespace) {
      list = list.filter((p) => p.namespace === activeNamespace)
    }
    const matched = q
      ? list.filter(
          (p) =>
            p.name.toLowerCase().includes(q) ||
            p.namespace.toLowerCase().includes(q) ||
            p.status.toLowerCase().includes(q) ||
            p.node.toLowerCase().includes(q),
        )
      : list
    return [...matched].sort((a, b) => {
      const cmp = compareValue(a, b, sortKey)
      return sortDir === 'asc' ? cmp : -cmp
    })
  }, [pods, activeNamespace, search, sortKey, sortDir])

  // ── Loading ────────────────────────────────────────────────────────────────

  if (isLoading) {
    return (
      <div className="flex flex-1 items-center justify-center text-text-muted text-xs">
        Loading pods…
      </div>
    )
  }

  // ── Error ──────────────────────────────────────────────────────────────────

  if (error) {
    return (
      <div className="flex flex-1 items-center justify-center text-error text-xs px-8 text-center">
        {String(error)}
      </div>
    )
  }

  // ── Table ──────────────────────────────────────────────────────────────────

  return (
    <div className="flex flex-col flex-1 overflow-hidden">

      {/* Search bar */}
      <div className="px-3 py-2 shrink-0 border-b border-border">
        <input
          type="text"
          placeholder="Filter pods…"
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className={cn(
            'w-64 h-6 px-2 bg-background border border-border rounded',
            'text-xs font-mono text-text-primary placeholder:text-text-muted',
            'focus:outline-none focus:border-accent',
          )}
        />
      </div>

      {/* Scrollable table */}
      <div className="flex-1 overflow-auto">
        <table className="w-full border-collapse text-left">
          <thead className="sticky top-0 z-10 bg-surface border-b border-border">
            <tr>
              {COLUMNS.map((col, i) => (
                <th
                  key={col.label || `col-${i}`}
                  onClick={() => handleSort(col.key)}
                  className={cn(
                    'px-3 py-1.5 text-xxs font-semibold tracking-wider text-text-muted uppercase select-none',
                    col.key && 'cursor-pointer hover:text-text-primary',
                    col.className,
                  )}
                >
                  <span className="flex items-center">
                    {col.label}
                    {col.key && <SortIcon col={col.key} sortKey={sortKey} sortDir={sortDir} />}
                  </span>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {filtered.length === 0 ? (
              <tr>
                <td
                  colSpan={COLUMNS.length}
                  className="px-3 py-8 text-center text-text-muted text-xs"
                >
                  {search ? 'No pods match your filter.' : 'No pods found in this namespace.'}
                </td>
              </tr>
            ) : (
              filtered.map((pod) => (
                <PodRow
                  key={`${pod.namespace}/${pod.name}`}
                  pod={pod}
                  isSelected={
                    selectedPod?.name === pod.name &&
                    selectedPod?.namespace === pod.namespace
                  }
                  onSelect={() => setSelectedPod(pod)}
                  onDescribe={() => { setSelectedPod(pod); openOutputPanel('describe') }}
                  onLogs={() => { setSelectedPod(pod); openOutputPanel('logs') }}
                />
              ))
            )}
          </tbody>
        </table>
      </div>
    </div>
  )
}
