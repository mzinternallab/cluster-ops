import { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useClusterStore } from '@/store/clusterStore'
import { useNamespaceStore } from '@/store/namespaceStore'
import { queryClient } from '@/lib/queryClient'
import type { KubeContext, ClusterHealth } from '@/types/kubernetes'

// ── Namespace loader ───────────────────────────────────────────────────────────
// Callable outside React components — accesses Zustand store via getState().

async function loadNamespaces() {
  const { setAvailableNamespaces, setActiveNamespace } = useNamespaceStore.getState()
  try {
    const namespaces = await invoke<string[]>('list_namespaces')
    setAvailableNamespaces(namespaces)
  } catch {
    setAvailableNamespaces([])
  }
  // Always reset filter so a stale namespace from the previous cluster isn't stuck
  setActiveNamespace(null)
}

// ── useCluster ─────────────────────────────────────────────────────────────────

/**
 * Initializes cluster state on app startup.
 * Call once at the root of the component tree (App.tsx).
 *
 * - Loads all kubeconfig contexts and marks the active one
 * - Kicks off concurrent health checks for every context that has a server URL
 * - Health results stream in independently via setHealth
 * - Loads namespace list for the active cluster
 */
export function useCluster() {
  const { setAvailableContexts, setActiveContext, setHealth } = useClusterStore()

  useEffect(() => {
    async function init() {
      const contexts = await invoke<KubeContext[]>('get_kubeconfig_contexts')
      setAvailableContexts(contexts)

      const active = contexts.find((c) => c.isActive)
      if (active) setActiveContext(active)

      // Health checks run concurrently — do not await in sequence
      for (const ctx of contexts) {
        if (!ctx.serverUrl) {
          setHealth(ctx.name, 'unknown')
          continue
        }
        invoke<string>('check_cluster_health', { serverUrl: ctx.serverUrl })
          .then((h) => setHealth(ctx.name, h as ClusterHealth))
          .catch(() => setHealth(ctx.name, 'unreachable'))
      }

      // Load namespace list for the active cluster
      await loadNamespaces()
    }

    init().catch(console.error)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])
}

// ── switchClusterContext ───────────────────────────────────────────────────────

/**
 * Switches to a different cluster context.
 * Persists the change to the kubeconfig file and updates the store.
 * Re-runs a health check for the newly active context.
 * Reloads the namespace list and invalidates the pod cache.
 */
export async function switchClusterContext(
  ctx: KubeContext,
  setActiveContext: (ctx: KubeContext) => void,
  setHealth: (name: string, health: ClusterHealth) => void,
) {
  // Persist the selection to the kubeconfig file.
  await invoke('set_active_context', { contextName: ctx.name })
  setActiveContext(ctx)

  // Kick off health check fire-and-forget.
  if (ctx.serverUrl) {
    setHealth(ctx.name, 'unknown')
    invoke<string>('check_cluster_health', { serverUrl: ctx.serverUrl })
      .then((h) => setHealth(ctx.name, h as ClusterHealth))
      .catch(() => setHealth(ctx.name, 'unreachable'))
  }

  // Restart proxy pointed at the single kubeconfig file that owns this context.
  // Wait for it to succeed before fetching namespaces or pods.
  await invoke('start_kubectl_proxy', {
    sourceFile: ctx.sourceFile,
    context: ctx.name,
  })

  await loadNamespaces()
  queryClient.invalidateQueries({ queryKey: ['pods'] })
}
