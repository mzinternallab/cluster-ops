import { create } from 'zustand'

interface NamespaceState {
  activeNamespace: string | null // null = all namespaces
  availableNamespaces: string[]
  podSearch: string
  setActiveNamespace: (ns: string | null) => void
  setAvailableNamespaces: (namespaces: string[]) => void
  setPodSearch: (search: string) => void
}

export const useNamespaceStore = create<NamespaceState>((set) => ({
  activeNamespace: null,
  availableNamespaces: [],
  podSearch: '',
  setActiveNamespace: (ns) => set({ activeNamespace: ns }),
  setAvailableNamespaces: (namespaces) => set({ availableNamespaces: namespaces }),
  setPodSearch: (search) => set({ podSearch: search }),
}))
