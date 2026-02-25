// AI analysis response types for cluster-ops

export type InsightType = 'critical' | 'warning' | 'suggestion'

export interface AIInsight {
  type: InsightType
  title: string
  body: string
  command?: string
}

export interface AIAnalysisResponse {
  insights: AIInsight[]
}

export type AIAnalysisMode = 'describe' | 'logs' | 'exec' | 'command'
