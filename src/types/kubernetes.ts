// Kubernetes resource types for cluster-ops

export type ClusterHealth = 'healthy' | 'slow' | 'unreachable' | 'unknown'

export interface KubeContext {
  name: string
  cluster: string
  user: string
  namespace?: string
  isActive: boolean
  /** API server URL — used for health checks; comes from the clusters stanza */
  serverUrl?: string
  /** Absolute path of the kubeconfig file that owns this context.
   *  Passed as --kubeconfig to kubectl proxy to avoid multi-path separator issues. */
  kubeconfigFile?: string
}

export interface PodSummary {
  name: string
  namespace: string
  status: string
  ready: string
  restarts: number
  age: string
  cpu: string
  memory: string
  node: string
  labels: Record<string, string>
}

export type PodStatus =
  | 'Running'
  | 'Pending'
  | 'Terminating'
  | 'CrashLoopBackOff'
  | 'OOMKilled'
  | 'Error'
  | 'Completed'
  | 'Unknown'
