import { useState, useRef, useEffect } from 'react'
import { ChevronDown, Search, X } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useNamespaceStore } from '@/store/namespaceStore'

export function NamespaceBar() {
  const { activeNamespace, availableNamespaces, setActiveNamespace, podSearch, setPodSearch } =
    useNamespaceStore()

  const [dropdownOpen, setDropdownOpen] = useState(false)
  const [nsFilter, setNsFilter] = useState('')
  const dropdownRef = useRef<HTMLDivElement>(null)

  // Close dropdown on outside click
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setDropdownOpen(false)
        setNsFilter('')
      }
    }
    if (dropdownOpen) {
      document.addEventListener('mousedown', handleClickOutside)
    }
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [dropdownOpen])

  const filteredNamespaces = availableNamespaces.filter((ns) =>
    ns.toLowerCase().includes(nsFilter.toLowerCase()),
  )

  function selectNamespace(ns: string | null) {
    setActiveNamespace(ns)
    setPodSearch('')
    setDropdownOpen(false)
    setNsFilter('')
  }

  return (
    <div className="flex items-center gap-2 px-3 h-9 shrink-0">

      {/* Namespace dropdown */}
      <div className="relative" ref={dropdownRef}>
        <button
          onClick={() => setDropdownOpen((o) => !o)}
          className={cn(
            'flex items-center justify-between gap-1 h-6 px-2 rounded',
            'bg-surface border border-border text-xxs font-mono text-text-primary',
            'hover:border-text-muted/40 focus:outline-none transition-colors',
          )}
          style={{ width: 220 }}
        >
          <span className="truncate">{activeNamespace ?? 'All Namespaces'}</span>
          <ChevronDown size={10} className="shrink-0 text-text-muted" />
        </button>

        {dropdownOpen && (
          <div
            className="absolute top-full left-0 mt-1 z-50 rounded border border-border bg-surface shadow-lg"
            style={{ width: 220 }}
          >
            {/* Filter input */}
            <div className="p-1.5 border-b border-border">
              <input
                autoFocus
                type="text"
                placeholder="Search namespaces..."
                value={nsFilter}
                onChange={(e) => setNsFilter(e.target.value)}
                className={cn(
                  'w-full h-6 px-2 bg-background border border-border rounded',
                  'text-xxs font-mono text-text-primary placeholder:text-text-muted',
                  'focus:outline-none focus:border-accent',
                )}
              />
            </div>

            {/* List */}
            <div className="max-h-48 overflow-y-auto py-1">
              <button
                onClick={() => selectNamespace(null)}
                className={cn(
                  'w-full text-left px-2 py-1 text-xxs font-mono truncate transition-colors',
                  'hover:bg-accent/10',
                  activeNamespace === null ? 'text-accent' : 'text-text-primary',
                )}
              >
                All Namespaces
              </button>

              {filteredNamespaces.map((ns) => (
                <button
                  key={ns}
                  onClick={() => selectNamespace(ns)}
                  className={cn(
                    'w-full text-left px-2 py-1 text-xxs font-mono truncate transition-colors',
                    'hover:bg-accent/10',
                    activeNamespace === ns ? 'text-accent' : 'text-text-muted',
                  )}
                >
                  {ns}
                </button>
              ))}

              {filteredNamespaces.length === 0 && nsFilter && (
                <div className="px-2 py-1 text-xxs font-mono text-text-muted/60">
                  No namespaces match
                </div>
              )}
            </div>
          </div>
        )}
      </div>

      {/* Pod search bar */}
      <div className="relative flex items-center" style={{ width: 240 }}>
        <Search size={11} className="absolute left-2 text-text-muted pointer-events-none shrink-0" />
        <input
          type="text"
          placeholder="Search pods..."
          value={podSearch}
          onChange={(e) => setPodSearch(e.target.value)}
          className={cn(
            'w-full h-6 pl-6 pr-5 bg-surface border border-border rounded',
            'text-xxs font-mono text-text-primary placeholder:text-text-muted',
            'focus:outline-none focus:border-accent transition-colors',
          )}
        />
        {podSearch && (
          <button
            onClick={() => setPodSearch('')}
            className="absolute right-1.5 text-text-muted hover:text-text-primary"
          >
            <X size={10} />
          </button>
        )}
      </div>

    </div>
  )
}
