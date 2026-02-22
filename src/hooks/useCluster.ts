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
      console.log('[useCluster] calling get_kubeconfig_contexts...')
      let contexts: KubeContext[]
      try {
        contexts = await invoke<KubeContext[]>('get_kubeconfig_contexts')
        console.log('[useCluster] raw response:', contexts)
        console.log('[useCluster] context count:', contexts.length)
        contexts.forEach((c) => console.log('[useCluster]  context:', c.name, 'isActive:', c.isActive))
      } catch (err) {
        console.error('[useCluster] get_kubeconfig_contexts ERROR:', err)
        return
      }
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
  await invoke('set_active_context', { contextName: ctx.name })
  setActiveContext(ctx)

  if (ctx.serverUrl) {
    setHealth(ctx.name, 'unknown') // show "checking" while pinging
    invoke<string>('check_cluster_health', { serverUrl: ctx.serverUrl })
      .then((h) => setHealth(ctx.name, h as ClusterHealth))
      .catch(() => setHealth(ctx.name, 'unreachable'))
  }

  // Reload namespaces for the new cluster and reset the namespace filter.
  // Then invalidate the pod cache so stale pods from the previous cluster
  // aren't served during the next polling interval.
  await loadNamespaces()
  queryClient.invalidateQueries({ queryKey: ['pods'] })
}
