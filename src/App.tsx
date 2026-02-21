import { TitleBar } from '@/components/layout/TitleBar'
import { Sidebar } from '@/components/layout/Sidebar'
import { NamespaceBar } from '@/components/layout/NamespaceBar'
import { CommandBar } from '@/components/terminal/CommandBar'
import { WorkloadsView } from '@/views/WorkloadsView'
import { useUIStore } from '@/store/uiStore'

export default function App() {
  const { outputPanelOpen } = useUIStore()

  return (
    // Root: full viewport, dark background, no overflow
    <div className="flex flex-col h-screen bg-background text-text-primary overflow-hidden select-none">

      {/* Custom title bar (draggable, window controls, cluster switcher) */}
      <TitleBar />

      {/* Main layout below title bar */}
      <div className="flex flex-1 overflow-hidden">

        {/* Icon-only left sidebar navigation */}
        <Sidebar />

        {/* Content area: namespace bar + main view + optional output panel */}
        <div className="flex flex-col flex-1 overflow-hidden">

          {/* Namespace filter bar + command bar */}
          <div className="flex items-center bg-surface border-b border-border shrink-0">
            <NamespaceBar />
            <div className="flex-1" />
            <CommandBar />
          </div>

          {/* Main content: workloads or other views */}
          <div className={`flex flex-col ${outputPanelOpen ? 'flex-[0_0_50%]' : 'flex-1'} overflow-hidden`}>
            <WorkloadsView />
          </div>

          {/* Output panel (slides up from bottom when describe/logs triggered) */}
          {outputPanelOpen && (
            <div className="flex flex-1 overflow-hidden border-t border-border">
              {/* Terminal output — Phase 1 Step 10 */}
              <div className="flex-1 overflow-hidden bg-background" />
              {/* AI analysis — Phase 1 Step 12 */}
            </div>
          )}

        </div>
      </div>
    </div>
  )
}
