import { Layers, FileText, Lock, Globe, Database } from 'lucide-react'
import type { LucideIcon } from 'lucide-react'
import { cn } from '@/lib/utils'
import { useUIStore } from '@/store/uiStore'

// Must stay in sync with ActiveView in uiStore
type ActiveView = 'workloads' | 'configmaps' | 'secrets' | 'network' | 'storage'

interface NavItem {
  view: ActiveView
  icon: LucideIcon
  label: string
}

const NAV_ITEMS: NavItem[] = [
  { view: 'workloads',  icon: Layers,    label: 'Workloads' },
  { view: 'configmaps', icon: FileText,  label: 'ConfigMaps' },
  { view: 'secrets',    icon: Lock,      label: 'Secrets' },
  { view: 'network',    icon: Globe,     label: 'Network' },
  { view: 'storage',    icon: Database,  label: 'Storage' },
]

export function Sidebar() {
  const { activeView, setActiveView } = useUIStore()

  return (
    <aside className="flex flex-col w-[52px] bg-surface border-r border-border shrink-0">
      {NAV_ITEMS.map(({ view, icon: Icon, label }) => {
        const isActive = activeView === view
        return (
          <button
            key={view}
            onClick={() => setActiveView(view)}
            title={label}
            aria-label={label}
            aria-current={isActive ? 'page' : undefined}
            className={cn(
              'flex items-center justify-center w-full h-11 border-l-2 transition-colors duration-100',
              'focus-visible:outline-none shrink-0',
              isActive
                ? 'border-accent text-accent bg-accent/10'
                : 'border-transparent text-text-muted hover:text-text-primary hover:bg-white/4',
            )}
          >
            <Icon size={16} strokeWidth={1.5} />
          </button>
        )
      })}
    </aside>
  )
}
