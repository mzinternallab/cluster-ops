import { TitleBar } from '@/components/layout/TitleBar'
import { Sidebar } from '@/components/layout/Sidebar'
import { NamespaceBar } from '@/components/layout/NamespaceBar'
import { CommandBar } from '@/components/terminal/CommandBar'
import { OutputPanel } from '@/components/terminal/OutputPanel'
import { WorkloadsView } from '@/views/WorkloadsView'
import { ConfigMapsView } from '@/views/ConfigMapsView'
import { SecretsView } from '@/views/SecretsView'
import { NetworkView } from '@/views/NetworkView'
import { StorageView } from '@/views/StorageView'
import { useUIStore } from '@/store/uiStore'
import { useCluster } from '@/hooks/useCluster'

function ActiveView() {
  const { activeView } = useUIStore()
  switch (activeView) {
    case 'workloads':  return <WorkloadsView />
    case 'configmaps': return <ConfigMapsView />
    case 'secrets':    return <SecretsView />
    case 'network':    return <NetworkView />
    case 'storage':    return <StorageView />
  }
}

export default function App() {
  const { outputPanelOpen } = useUIStore()
  // Load kubeconfig contexts and kick off health checks on startup
  useCluster()

  return (
    <div className="flex flex-col h-screen bg-background text-text-primary overflow-hidden select-none">

      <TitleBar />

      <div className="flex flex-1 overflow-hidden">

        <Sidebar />

        <div className="flex flex-col flex-1 overflow-hidden">

          {/* Namespace filter bar + kubectl command bar */}
          <div className="flex items-center bg-surface border-b border-border shrink-0">
            <NamespaceBar />
            <div className="flex-1" />
            <CommandBar />
          </div>

          {/* Main content area — shrinks to 50% when output panel opens */}
          <div
            className={`flex flex-col overflow-hidden ${
              outputPanelOpen ? 'flex-[0_0_50%]' : 'flex-1'
            }`}
          >
            <ActiveView />
          </div>

          {/* Bottom split: output panel + AI panel (Step 10 + 12) */}
          {outputPanelOpen && (
            <div className="flex flex-1 overflow-hidden border-t border-border">
              <OutputPanel />
              {/* AI panel — Step 12 */}
            </div>
          )}

        </div>
      </div>
    </div>
  )
}
