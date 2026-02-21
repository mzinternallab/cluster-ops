import { create } from 'zustand'

interface NamespaceState {
  activeNamespace: string | null // null = all namespaces
  availableNamespaces: string[]
  setActiveNamespace: (ns: string | null) => void
  setAvailableNamespaces: (namespaces: string[]) => void
}

export const useNamespaceStore = create<NamespaceState>((set) => ({
  activeNamespace: null,
  availableNamespaces: [],
  setActiveNamespace: (ns) => set({ activeNamespace: ns }),
  setAvailableNamespaces: (namespaces) => set({ availableNamespaces: namespaces }),
}))
