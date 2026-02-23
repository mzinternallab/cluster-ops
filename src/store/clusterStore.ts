import { create } from 'zustand'
import type { KubeContext, ClusterHealth } from '@/types/kubernetes'

interface ClusterState {
  activeContext: KubeContext | null
  availableContexts: KubeContext[]
  healthMap: Record<string, ClusterHealth> // contextName -> health
  proxyReady: boolean
  setActiveContext: (ctx: KubeContext) => void
  setAvailableContexts: (contexts: KubeContext[]) => void
  setHealth: (contextName: string, health: ClusterHealth) => void
  setProxyReady: (ready: boolean) => void
}

export const useClusterStore = create<ClusterState>((set) => ({
  activeContext: null,
  availableContexts: [],
  healthMap: {},
  proxyReady: false,
  setActiveContext: (ctx) => set({ activeContext: ctx }),
  setAvailableContexts: (contexts) => set({ availableContexts: contexts }),
  setHealth: (contextName, health) =>
    set((s) => ({ healthMap: { ...s.healthMap, [contextName]: health } })),
  setProxyReady: (ready) => set({ proxyReady: ready }),
}))
