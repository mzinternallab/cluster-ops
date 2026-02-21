// Pods + Deployments + ReplicaSets view — Phase 1 Step 7-8
import { PodTable } from '@/components/workloads/PodTable'

export function WorkloadsView() {
  return (
    <div className="flex flex-col flex-1 overflow-hidden">
      <PodTable />
    </div>
  )
}
