import { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useClusterStore } from '@/store/clusterStore'
import { useNamespaceStore } from '@/store/namespaceStore'
import { queryClient } from '@/lib/queryClient'
import type { KubeContext, ClusterHealth } from '@/types/kubernetes'

// ── Namespace loader ───────────────────────────────────────────────────────────

async function loadNamespaces() {
  const { setAvailableNamespaces, setActiveNamespace } = useNamespaceStore.getState()
  try {
    const namespaces = await invoke<string[]>('list_namespaces')
    setAvailableNamespaces(namespaces)
  } catch {
    setAvailableNamespaces([])
  }
  setActiveNamespace(null)
}

// ── useCluster ─────────────────────────────────────────────────────────────────

export function useCluster() {
  const { setAvailableContexts, setActiveContext, setHealth } = useClusterStore()

  useEffect(() => {
    async function init() {
      const contexts = await invoke<KubeContext[]>('get_kubeconfig_contexts')
      setAvailableContexts(contexts)

      const active = contexts.find((c) => c.isActive)
      if (active) setActiveContext(active)

      // Health checks run concurrently — keyed by displayName (unique per cluster).
      for (const ctx of contexts) {
        if (!ctx.serverUrl) {
          setHealth(ctx.displayName, 'unknown')
          continue
        }
        invoke<string>('check_cluster_health', { serverUrl: ctx.serverUrl })
          .then((h) => setHealth(ctx.displayName, h as ClusterHealth))
          .catch(() => setHealth(ctx.displayName, 'unreachable'))
      }

      await loadNamespaces()
    }

    init().catch(() => {})
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])
}

// ── switchClusterContext ───────────────────────────────────────────────────────

export async function switchClusterContext(
  ctx: KubeContext,
  setActiveContext: (ctx: KubeContext) => void,
  setHealth: (name: string, health: ClusterHealth) => void,
) {
  // Persist the selection to the specific source file that owns this context.
  await invoke('set_active_context', {
    contextName: ctx.contextName,
    sourceFile: ctx.sourceFile,
  })
  setActiveContext(ctx)

  // Health check keyed by displayName (unique per cluster).
  if (ctx.serverUrl) {
    setHealth(ctx.displayName, 'unknown')
    invoke<string>('check_cluster_health', { serverUrl: ctx.serverUrl })
      .then((h) => setHealth(ctx.displayName, h as ClusterHealth))
      .catch(() => setHealth(ctx.displayName, 'unreachable'))
  }

  // Restart proxy with the single kubeconfig file and exact context name.
  await invoke('start_kubectl_proxy', {
    sourceFile: ctx.sourceFile,
    contextName: ctx.contextName,
  })

  await loadNamespaces()
  queryClient.invalidateQueries({ queryKey: ['pods'] })
}
