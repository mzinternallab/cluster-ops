import { useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { useClusterStore } from '@/store/clusterStore'
import type { KubeContext, ClusterHealth } from '@/types/kubernetes'

/**
 * Initializes cluster state on app startup.
 * Call once at the root of the component tree (App.tsx).
 *
 * - Loads all kubeconfig contexts and marks the active one
 * - Kicks off concurrent health checks for every context that has a server URL
 * - Health results stream in independently via setHealth
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
    }

    init().catch(console.error)
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])
}

/**
 * Switches to a different cluster context.
 * Persists the change to the kubeconfig file and updates the store.
 * Re-runs a health check for the newly active context.
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
}
