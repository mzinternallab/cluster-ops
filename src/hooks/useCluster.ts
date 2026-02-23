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
 * Call once at the root of the component tree (App.tsx → AppContent).
 * By the time this runs, App.tsx has already started kubectl proxy and
 * set proxyReady = true, so API calls to :8001 are safe.
 *
 * - Loads all kubeconfig contexts and marks the active one
 * - Kicks off concurrent health checks for every context that has a server URL
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
 *
 * 1. Shows the "Connecting to cluster…" screen by setting proxyReady = false.
 * 2. Restarts kubectl proxy with the new context (Rust sleeps 1500 ms before
 *    returning, so the proxy is listening by the time we proceed).
 * 3. Reloads the namespace list and invalidates the pod cache.
 * 4. Marks the proxy as ready again so the main UI is shown.
 *
 * set_active_context writes current-context to the primary kubeconfig file
 * (best-effort — non-fatal if the file doesn't exist on the current system).
 */
export async function switchClusterContext(
  ctx: KubeContext,
  setActiveContext: (ctx: KubeContext) => void,
  setHealth: (name: string, health: ClusterHealth) => void,
) {
  const { setProxyReady } = useClusterStore.getState()

  // Show "Connecting to cluster…" while the proxy restarts.
  setProxyReady(false)

  try {
    // Persist the context choice to the kubeconfig file (best-effort).
    invoke('set_active_context', { contextName: ctx.name }).catch(() => {
      // Non-fatal: the file may not exist on this machine.
    })

    // Restart kubectl proxy pointed at the new context.
    // The Rust command kills the old proxy, spawns a new one, and waits
    // 1500 ms for it to start before resolving.
    await invoke('start_kubectl_proxy', { context: ctx.name })

    setActiveContext(ctx)

    // Re-ping health for the newly active context.
    if (ctx.serverUrl) {
      setHealth(ctx.name, 'unknown')
      invoke<string>('check_cluster_health', { serverUrl: ctx.serverUrl })
        .then((h) => setHealth(ctx.name, h as ClusterHealth))
        .catch(() => setHealth(ctx.name, 'unreachable'))
    }

    // Reload namespaces for the new cluster and reset the namespace filter.
    // Invalidate the pod cache so stale pods from the previous cluster are gone.
    await loadNamespaces()
    queryClient.invalidateQueries({ queryKey: ['pods'] })
  } finally {
    // Always restore the UI — even if something above threw.
    setProxyReady(true)
  }
}
