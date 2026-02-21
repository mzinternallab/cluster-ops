import { cn } from '@/lib/utils'
import { useNamespaceStore } from '@/store/namespaceStore'

export function NamespaceBar() {
  const { activeNamespace, availableNamespaces, setActiveNamespace } = useNamespaceStore()

  // null = "all namespaces" sentinel
  const pills: (string | null)[] = [null, ...availableNamespaces]

  return (
    <div className="flex items-center gap-1 px-3 h-9 overflow-x-auto scrollbar-none shrink-0">
      {pills.map((ns) => {
        const isActive = activeNamespace === ns
        const label = ns ?? 'all'

        return (
          <button
            key={label}
            onClick={() => setActiveNamespace(ns)}
            className={cn(
              'flex items-center h-5 px-2 rounded text-xxs font-mono whitespace-nowrap',
              'transition-colors duration-100 border focus-visible:outline-none',
              isActive
                ? 'bg-accent/15 text-accent border-accent/40'
                : 'text-text-muted border-border hover:text-text-primary hover:border-text-muted/40',
            )}
          >
            {label}
          </button>
        )
      })}
    </div>
  )
}
