// Pod list fetching and polling
// Implemented in Phase 1 Step 6-8

export function usePods(_namespace?: string | null) {
  // TODO: useQuery -> invoke('list_pods', { namespace })
  return { pods: [], isLoading: false, error: null }
}
