import { create } from 'zustand'
import type { KubeContext, ClusterHealth } from '@/types/kubernetes'

interface ClusterState {
  activeContext: KubeContext | null
  availableContexts: KubeContext[]
  healthMap: Record<string, ClusterHealth> // contextName -> health
  setActiveContext: (ctx: KubeContext) => void
  setAvailableContexts: (contexts: KubeContext[]) => void
  setHealth: (contextName: string, health: ClusterHealth) => void
}

export const useClusterStore = create<ClusterState>((set) => ({
  activeContext: null,
  availableContexts: [],
  healthMap: {},
  setActiveContext: (ctx) => set({ activeContext: ctx }),
  setAvailableContexts: (contexts) => set({ availableContexts: contexts }),
  setHealth: (contextName, health) =>
    set((s) => ({ healthMap: { ...s.healthMap, [contextName]: health } })),
}))
