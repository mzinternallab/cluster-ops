import { useQuery } from '@tanstack/react-query'
import { invoke } from '@tauri-apps/api/core'
import { useNamespaceStore } from '@/store/namespaceStore'
import type { PodSummary } from '@/types/kubernetes'

export function usePods() {
  const { activeNamespace } = useNamespaceStore()

  return useQuery({
    queryKey: ['pods', activeNamespace ?? 'all'],
    queryFn: () => invoke<PodSummary[]>('list_pods', { namespace: activeNamespace }),
    refetchInterval: 10_000,
  })
}
