import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { useNamespaceStore } from '@/store/namespaceStore'
import type { PodSummary } from '@/types/kubernetes'

export function usePods() {
  // Selector subscription — only re-renders when activeNamespace changes
  const activeNamespace = useNamespaceStore((s) => s.activeNamespace)

  return useQuery({
    queryKey: ['pods', activeNamespace ?? 'all'],
    queryFn: () => invoke<PodSummary[]>('list_pods', { namespace: activeNamespace }),
    refetchInterval: 10_000,
    // Keep the previous namespace's pod list visible while the new one loads.
    // In TanStack Query v5 this sets status → 'success' so isLoading stays false,
    // which prevents the table from being replaced with "Loading pods…" on every switch.
    placeholderData: keepPreviousData,
  })
}
