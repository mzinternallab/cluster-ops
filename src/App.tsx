import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { KubeContext } from '@/types/kubernetes'
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

// Rendered only after the kubectl proxy is confirmed up.
// useCluster() is gated here so no API calls fire before :8001 is listening.
function AppContent() {
  const { outputPanelOpen } = useUIStore()
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

export default function App() {
  const [proxyReady, setProxyReady] = useState(false)
  const [proxyError, setProxyError] = useState<string | null>(null)

  useEffect(() => {
    const cleanup = () => { invoke('stop_kubectl_proxy').catch(() => {}) }
    window.addEventListener('beforeunload', cleanup)

    async function init() {
      // Load contexts first so we can pass the active context's source file to
      // the proxy — a single-file --kubeconfig avoids path-separator issues.
      let sourceFile: string | undefined
      let context: string | undefined
      try {
        const contexts = await invoke<KubeContext[]>('get_kubeconfig_contexts')
        const active = contexts.find((c) => c.isActive)
        sourceFile = active?.sourceFile
        context = active?.name
      } catch {
        // Non-fatal: proxy will start without --kubeconfig / --context.
      }

      await invoke('start_kubectl_proxy', { sourceFile, context })
      // Give kubectl proxy ~400 ms to start listening on :8001.
      setTimeout(() => setProxyReady(true), 400)
    }

    init().catch((e: unknown) => setProxyError(String(e)))
    return () => window.removeEventListener('beforeunload', cleanup)
  }, [])

  if (proxyError) {
    return (
      <div className="flex flex-col h-screen bg-background text-text-primary overflow-hidden select-none">
        <TitleBar />
        <div className="flex flex-1 items-center justify-center text-error text-xs font-mono px-8 text-center">
          kubectl proxy failed to start: {proxyError}
        </div>
      </div>
    )
  }

  if (!proxyReady) {
    return (
      <div className="flex flex-col h-screen bg-background text-text-primary overflow-hidden select-none">
        <TitleBar />
        <div className="flex flex-1 items-center justify-center text-text-muted text-xs font-mono">
          Connecting to cluster…
        </div>
      </div>
    )
  }

  return <AppContent />
}
