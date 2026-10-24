import type { GraphEdge, NodeLabel, PathNode, PathShape } from '../types'

export const LABEL_COLORS: Record<NodeLabel, string> = {
  Person: '#38bdf8',
  Repo: '#c084fc',
  Issue: '#fb7185',
  Topic: '#4ade80',
  Talk: '#fbbf24',
  Encounter: '#f472b6',
}

export const LABEL_RGB: Record<NodeLabel, string> = {
  Person: '56,189,248',
  Repo: '192,132,252',
  Issue: '251,113,133',
  Topic: '74,222,128',
  Talk: '251,191,36',
  Encounter: '244,114,182',
}

export function labelColor(label: string): string {
  return LABEL_COLORS[label as NodeLabel] ?? '#94a3b8'
}

export function labelRgb(label: string): string {
  return LABEL_RGB[label as NodeLabel] ?? '148,163,184'
}

const AGENT_COLORS: Record<string, string> = {
  Cartographer: '#38bdf8',
  Pathfinder: '#4ade80',
  Concierge: '#f472b6',
}

export function agentColor(agent: string): string {
  return AGENT_COLORS[agent] ?? '#94a3b8'
}

const IN_PROGRESS_STATES = new Set(['SYNCING', 'CLASSIFYING', 'LINKING', 'DRAFTING', 'PARSING'])

export function isInProgress(state?: string): boolean {
  return !!state && IN_PROGRESS_STATES.has(state)
}

export function edgeKey(type: string, srcUid: string, dstUid: string): string {
  return `${type}|${srcUid}|${dstUid}`
}

export function graphEdgeKey(e: GraphEdge): string {
  return edgeKey(e.type, e.src_uid, e.dst_uid)
}

export interface Highlight {
  nodes: Set<string>
  edges: Set<string>
  nonce: number
}

export function highlightFromPath(path: PathShape, nonce: number): Highlight {
  return {
    nodes: new Set(path.nodes.map((n) => n.uid)),
    edges: new Set(path.edges.map(graphEdgeKey)),
    nonce,
  }
}

export function degreesOf(edges: GraphEdge[]): Map<string, number> {
  const deg = new Map<string, number>()
  for (const e of edges) {
    deg.set(e.src_uid, (deg.get(e.src_uid) ?? 0) + 1)
    deg.set(e.dst_uid, (deg.get(e.dst_uid) ?? 0) + 1)
  }
  return deg
}

export interface OrderedPathNode {
  node: PathNode
  via: GraphEdge | null
}

/**
 * Walk the returned path as an undirected chain starting from its first node,
 * so badges read Me › Repo › Issue › Topic › Talk › Speaker.
 */
export function pathOrder(path: PathShape): OrderedPathNode[] {
  const byUid = new Map(path.nodes.map((n) => [n.uid, n]))
  const adj = new Map<string, { to: string; edge: GraphEdge }[]>()
  const link = (from: string, to: string, edge: GraphEdge) => {
    const list = adj.get(from) ?? []
    list.push({ to, edge })
    adj.set(from, list)
  }
  for (const e of path.edges) {
    if (!byUid.has(e.src_uid) || !byUid.has(e.dst_uid)) continue
    link(e.src_uid, e.dst_uid, e)
    link(e.dst_uid, e.src_uid, e)
  }
  const out: OrderedPathNode[] = []
  const seen = new Set<string>()
  const via = new Map<string, GraphEdge>()
  const queue: string[] = path.nodes[0] ? [path.nodes[0].uid] : []
  while (queue.length > 0) {
    const uid = queue.shift()
    if (!uid || seen.has(uid)) continue
    seen.add(uid)
    const node = byUid.get(uid)
    if (node) out.push({ node, via: via.get(uid) ?? null })
    for (const { to, edge } of adj.get(uid) ?? []) {
      if (!seen.has(to)) {
        via.set(to, edge)
        queue.push(to)
      }
    }
  }
  return out
}

export function fmtClock(ts: number): string {
  return new Date(ts).toLocaleTimeString([], { hour12: false })
}
