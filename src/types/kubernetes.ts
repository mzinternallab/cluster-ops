// Kubernetes resource types for cluster-ops

export type ClusterHealth = 'healthy' | 'slow' | 'unreachable' | 'unknown'

export interface KubeContext {
  /** Derived from the kubeconfig filename — shown in the UI.
   *  e.g. "config.eagle-i-orc" → "eagle-i-orc" */
  displayName: string
  /** Actual context name inside the kubeconfig file, e.g. "local".
   *  Always passed as --context to kubectl subprocesses. */
  contextName: string
  /** Absolute path to the kubeconfig file that owns this context.
   *  Always passed as --kubeconfig to kubectl subprocesses. */
  sourceFile: string
  cluster: string
  user: string
  isActive: boolean
  /** API server URL — used for health checks */
  serverUrl?: string
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
