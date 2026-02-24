import { create } from 'zustand'
import type { PodSummary } from '@/types/kubernetes'
import type { AIAnalysisMode } from '@/types/ai'

type ActiveView = 'workloads' | 'configmaps' | 'secrets' | 'network' | 'storage'

interface UIState {
  activeView: ActiveView
  selectedPod: PodSummary | null
  outputPanelOpen: boolean
  outputPanelMode: AIAnalysisMode | null
  aiPanelVisible: boolean
  execSessionKey: number
  setActiveView: (view: ActiveView) => void
  setSelectedPod: (pod: PodSummary | null) => void
  openOutputPanel: (mode: AIAnalysisMode) => void
  closeOutputPanel: () => void
  toggleAIPanel: () => void
  incrementExecSessionKey: () => void
}

export const useUIStore = create<UIState>((set) => ({
  activeView: 'workloads',
  selectedPod: null,
  outputPanelOpen: false,
  outputPanelMode: null,
  aiPanelVisible: true,
  execSessionKey: 0,
  setActiveView: (view) => set({ activeView: view }),
  setSelectedPod: (pod) => set({ selectedPod: pod }),
  openOutputPanel: (mode) => set({ outputPanelOpen: true, outputPanelMode: mode }),
  closeOutputPanel: () => set({ outputPanelOpen: false, outputPanelMode: null }),
  toggleAIPanel: () => set((s) => ({ aiPanelVisible: !s.aiPanelVisible })),
  incrementExecSessionKey: () => set((s) => ({ execSessionKey: s.execSessionKey + 1 })),
}))
