import { create } from 'zustand'
import type { KubeContext } from '@/types/kubernetes'

interface ClusterState {
  activeContext: KubeContext | null
  availableContexts: KubeContext[]
  setActiveContext: (ctx: KubeContext) => void
  setAvailableContexts: (contexts: KubeContext[]) => void
}

export const useClusterStore = create<ClusterState>((set) => ({
  activeContext: null,
  availableContexts: [],
  setActiveContext: (ctx) => set({ activeContext: ctx }),
  setAvailableContexts: (contexts) => set({ availableContexts: contexts }),
}))
