import healthJson from './mocks/health.json'
import syncJson from './mocks/sync.json'
import askJson from './mocks/ask.json'
import graphJson from './mocks/graph.json'
import eventsJson from './mocks/events.json'
import alertsJson from './mocks/alerts.json'
import alertSeenJson from './mocks/alert-seen.json'
import topicsJson from './mocks/topics.json'
import encountersJson from './mocks/encounters.json'
import injectIssueJson from './mocks/inject-issue.json'
import type {
  AgentEvent,
  Alert,
  AskEnvelope,
  GraphData,
  GraphEdge,
  GraphNode,
  HealthStatus,
  TopicItem,
} from './types'

export const IS_MOCK = import.meta.env.VITE_MOCK === '1'
export const PRESENTER_HANDLE = 'manvendersingh21'
export const PRESENTER_UID = 'person:gh:manvendersingh21'
export const REPO_UID = 'repo:manvendersingh21/deepcore-n1'

/** Act 3 storyline issue (PLAN.md seed data plan), used by the Alt+I fallback. */
export const ACT3_ISSUE = {
  title: 'Agents can merge PRs through our MCP server; we need per-identity scopes',
  body: 'Our hosted MCP server shares one OAuth identity across every agent. We need per-identity, per-tool scopes so agents can open PRs but never merge them.',
}

async function getJson<T>(url: string): Promise<T> {
  const res = await fetch(url)
  if (!res.ok) throw new Error(`GET ${url} failed: ${res.status}`)
  return (await res.json()) as T
}

async function postJson<T>(url: string, body: unknown): Promise<T> {
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) throw new Error(`POST ${url} failed: ${res.status}`)
  return (await res.json()) as T
}

// ---------------------------------------------------------------------------
// Mock mode: fixtures plus a tiny in-memory simulation of the agent pipeline,
// so the UI behaves like the real backend while running against fixtures.
// ---------------------------------------------------------------------------

const mockGraph: GraphData = JSON.parse(JSON.stringify(graphJson)) as GraphData
const mockEvents: AgentEvent[] = (eventsJson as unknown as AgentEvent[]).slice()
let mockClock = mockEvents.reduce((max, e) => Math.max(max, e.ts), 0)
let mockSeq = 0
const mockStartedAt = Date.now()
const seenBridges = new Set<string>()
/** The alert slides in shortly after load so Act 1 has the stage first. */
const MOCK_ALERT_DELAY_MS = 6000

function pushEvent(agent: string, action: string, target: string, detail: string): void {
  mockClock += 1000
  mockSeq += 1
  mockEvents.push({ uid: `evt:mock-${mockSeq}`, ts: mockClock, agent, action, target, detail })
}

function addNode(node: GraphNode): void {
  mockGraph.nodes.push(node)
}

function addEdge(edge: GraphEdge): void {
  const exists = mockGraph.edges.some(
    (e) => e.type === edge.type && e.src_uid === edge.src_uid && e.dst_uid === edge.dst_uid,
  )
  if (!exists) mockGraph.edges.push(edge)
}

function setNodeState(uid: string, state: string): void {
  const node = mockGraph.nodes.find((n) => n.uid === uid)
  if (node) node.state = state
}

function simulateInjectIssue(title: string, body: string): void {
  const uid = 'issue:manvendersingh21/deepcore-n1#12'
  if (mockGraph.nodes.some((n) => n.uid === uid)) return
  addNode({ uid, label: 'Issue', name: title, state: 'NEW', gh_state: 'open' })
  addEdge({ type: 'HAS_ISSUE', src_uid: REPO_UID, dst_uid: uid })
  pushEvent('Cartographer', 'inject', 'Issue #12', `NEW · Wi-Fi fallback · ${body.slice(0, 24)}…`)
  setTimeout(() => {
    setNodeState(uid, 'CLASSIFYING')
    pushEvent('Cartographer', 'classify', 'Issue #12', 'NEW → CLASSIFYING')
  }, 1200)
  setTimeout(() => {
    setNodeState(uid, 'LINKED')
    addEdge({
      type: 'ABOUT',
      src_uid: uid,
      dst_uid: 'topic:mcp-auth',
      confidence: 0.92,
      evidence: 'per-identity scopes for MCP tool calls',
    })
    addEdge({ type: 'BRIDGE', src_uid: uid, dst_uid: 'talk:U26-09', kind: 'talk', state: 'ACTION_READY' })
    pushEvent('Pathfinder', 'link', 'Issue #12', 'CLASSIFYING → LINKED')
    pushEvent('Concierge', 'draft', 'BRIDGE → talk:U26-09', 'DRAFTING → ACTION_READY')
  }, 3200)
}

function simulateEncounter(note: string, place: string): void {
  mockSeq += 1
  const encUid = `enc:mock-${mockSeq}`
  const contactUid = 'person:name:riley-chen'
  addNode({
    uid: encUid,
    label: 'Encounter',
    name: note.slice(0, 60),
    state: 'NEW',
    note,
    place,
    met_at: new Date().toISOString(),
  })
  addEdge({ type: 'LOGGED', src_uid: PRESENTER_UID, dst_uid: encUid })
  pushEvent('API', 'encounter', encUid, `NEW · ${place}`)
  setTimeout(() => {
    setNodeState(encUid, 'PARSING')
    pushEvent('Cartographer', 'parse', encUid, 'NEW → PARSING')
  }, 1000)
  setTimeout(() => {
    setNodeState(encUid, 'PARSED')
    addNode({ uid: contactUid, label: 'Person', name: 'Riley Chen', kind: 'contact' })
    addEdge({ type: 'MET', src_uid: encUid, dst_uid: contactUid })
    addEdge({
      type: 'DISCUSSED',
      src_uid: encUid,
      dst_uid: 'topic:mcp-auth',
      confidence: 0.86,
      evidence: 'scope agent tokens so agents can open PRs but never merge',
    })
    pushEvent('Cartographer', 'parse', encUid, 'PARSING → PARSED · +MET Riley Chen')
  }, 2200)
  setTimeout(() => {
    setNodeState(encUid, 'LINKED')
    pushEvent('Pathfinder', 'bridge', 'Riley Chen', 'PARSED → LINKED · BRIDGE DISCOVERED')
    pushEvent('Concierge', 'draft', 'BRIDGE → Riley Chen', 'DRAFTING → ACTION_READY')
  }, 3400)
}

function snapshotGraph(): GraphData {
  return {
    nodes: mockGraph.nodes.map((n) => ({ ...n })),
    edges: mockGraph.edges.map((e) => ({ ...e })),
  }
}

export const api = {
  health: (): Promise<HealthStatus> =>
    IS_MOCK ? Promise.resolve(healthJson as unknown as HealthStatus) : getJson('/api/health'),

  sync: (handle: string): Promise<{ person_uid: string }> =>
    IS_MOCK
      ? Promise.resolve(syncJson as unknown as { person_uid: string })
      : postJson('/api/sync', { handle, kind: 'self' }),

  ask: (question: string): Promise<AskEnvelope> =>
    IS_MOCK
      ? Promise.resolve(askJson as unknown as AskEnvelope)
      : postJson('/api/ask', { question, person_uid: PRESENTER_UID }),

  graph: (): Promise<GraphData> =>
    IS_MOCK ? Promise.resolve(snapshotGraph()) : getJson<GraphData>('/api/graph?scope=overview'),

  events: (since: number): Promise<AgentEvent[]> =>
    IS_MOCK
      ? Promise.resolve(mockEvents.filter((e) => e.ts > since))
      : getJson<AgentEvent[]>(`/api/events?since=${since}`),

  alerts: (): Promise<Alert[]> =>
    IS_MOCK
      ? Promise.resolve(
          (Date.now() - mockStartedAt >= MOCK_ALERT_DELAY_MS
            ? (alertsJson as unknown as Alert[])
            : []
          ).filter((a) => !seenBridges.has(a.bridge_uid)),
        )
      : getJson<Alert[]>('/api/alerts'),

  markAlertSeen: (bridgeUid: string): Promise<{ ok: boolean }> => {
    seenBridges.add(bridgeUid)
    return IS_MOCK
      ? Promise.resolve(alertSeenJson as unknown as { ok: boolean })
      : postJson(`/api/alerts/${bridgeUid}/seen`, {})
  },

  topics: (): Promise<TopicItem[]> =>
    IS_MOCK ? Promise.resolve(topicsJson as unknown as TopicItem[]) : getJson<TopicItem[]>('/api/topics'),

  encounter: (note: string, place: string): Promise<{ encounter_uid: string }> => {
    if (IS_MOCK) {
      simulateEncounter(note, place)
      return Promise.resolve(encountersJson as unknown as { encounter_uid: string })
    }
    return postJson('/api/encounters', { note, place })
  },

  injectIssue: (title: string, body: string): Promise<{ issue_uid: string }> => {
    if (IS_MOCK) {
      simulateInjectIssue(title, body)
      return Promise.resolve(injectIssueJson as unknown as { issue_uid: string })
    }
    return postJson('/api/demo/inject-issue', { title, body })
  },
}
