export type NodeLabel = 'Person' | 'Repo' | 'Issue' | 'Topic' | 'Talk' | 'Encounter'

export type EdgeType =
  | 'MAINTAINS'
  | 'HAS_ISSUE'
  | 'ABOUT'
  | 'COVERS'
  | 'SPEAKS_AT'
  | 'LOGGED'
  | 'MET'
  | 'DISCUSSED'
  | 'MENTIONED_REPO'
  | 'BRIDGE'

export interface GraphNode {
  uid: string
  label: NodeLabel
  name: string
  state?: string
  kind?: string
  handle?: string
  company?: string
  gh_state?: string
  day?: string
  start_time?: string
  end_time?: string
  room?: string
  note?: string
  place?: string
  met_at?: string
}

export interface GraphEdge {
  type: EdgeType
  src_uid: string
  dst_uid: string
  state?: string
  kind?: string
  confidence?: number
  evidence?: string
}

export interface GraphData {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export interface PathNode {
  uid: string
  label: NodeLabel
  name: string
  state?: string
}

export interface PathShape {
  nodes: PathNode[]
  edges: GraphEdge[]
}

export interface AskRow {
  issue?: string
  topic?: string
  talk?: string
  day?: string
  start_time?: string
  room?: string
  speaker?: string
  person?: string
  company?: string
  contact?: string
  note?: string
  place?: string
  met_at?: string
}

export interface AskEnvelope {
  template: string
  params: Record<string, unknown>
  cypher: string
  hops: number
  run_time_ms: number
  rows: AskRow[]
  path: PathShape
  answer: string
}

export interface HealthStatus {
  falkordb: string
  openai: string
  github: string
  counts: Record<string, number>
}

export interface AgentEvent {
  uid: string
  ts: number
  agent: string
  action: string
  target: string
  detail: string
}

export interface Alert {
  bridge_uid: string
  issue: string
  contact: string
  note: string
  place: string
  met_at: string
  draft: string
  path: PathShape
  cypher: string
  run_time_ms: number
}

export interface TopicItem {
  slug: string
  name: string
}
