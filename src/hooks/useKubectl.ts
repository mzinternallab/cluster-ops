// Run kubectl commands and stream output
// Implemented in Phase 1 Step 9-10

export function useKubectl() {
  // TODO: invoke('describe_pod'), invoke('get_pod_logs'), invoke('run_kubectl')
  return { run: (_cmd: string) => Promise.resolve('') }
}
