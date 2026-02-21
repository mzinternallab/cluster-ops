// Kubernetes resource types for cluster-ops

export interface KubeContext {
  name: string
  cluster: string
  user: string
  namespace?: string
  isActive: boolean
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
