import { create } from 'zustand'
import type { PodSummary } from '@/types/kubernetes'
import type { AIAnalysisMode } from '@/types/ai'

type ActiveView = 'workloads' | 'configmaps' | 'secrets' | 'network' | 'storage'

const MAX_HISTORY = 100

interface UIState {
  activeView: ActiveView
  selectedPod: PodSummary | null
  outputPanelOpen: boolean
  outputPanelMode: AIAnalysisMode | null
  aiPanelVisible: boolean
  execSessionKey: number
  commandKey: number        // increments on each kubectl command run
  commandHistory: string[]  // most recent last; capped at MAX_HISTORY
  setActiveView: (view: ActiveView) => void
  setSelectedPod: (pod: PodSummary | null) => void
  openOutputPanel: (mode: AIAnalysisMode) => void
  closeOutputPanel: () => void
  toggleAIPanel: () => void
  incrementExecSessionKey: () => void
  incrementCommandKey: () => void
  addToCommandHistory: (cmd: string) => void
}

export const useUIStore = create<UIState>((set) => ({
  activeView: 'workloads',
  selectedPod: null,
  outputPanelOpen: false,
  outputPanelMode: null,
  aiPanelVisible: true,
  execSessionKey: 0,
  commandKey: 0,
  commandHistory: [],
  setActiveView: (view) => set({ activeView: view }),
  setSelectedPod: (pod) => set({ selectedPod: pod }),
  openOutputPanel: (mode) => set({ outputPanelOpen: true, outputPanelMode: mode }),
  closeOutputPanel: () => set({ outputPanelOpen: false, outputPanelMode: null }),
  toggleAIPanel: () => set((s) => ({ aiPanelVisible: !s.aiPanelVisible })),
  incrementExecSessionKey: () => set((s) => ({ execSessionKey: s.execSessionKey + 1 })),
  incrementCommandKey: () => set((s) => ({ commandKey: s.commandKey + 1 })),
  addToCommandHistory: (cmd) =>
    set((s) => {
      const filtered = s.commandHistory.filter((c) => c !== cmd)
      const next = [...filtered, cmd]
      return { commandHistory: next.slice(-MAX_HISTORY) }
    }),
}))
