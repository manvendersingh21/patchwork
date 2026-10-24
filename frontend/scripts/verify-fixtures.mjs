// Verifies the mock fixtures against each other: every path node/edge used by
// the ask and alerts fixtures must resolve inside the graph fixture, the Act 1
// chain must be intact, and the ask envelope must match the API contract.
import { readFileSync } from 'node:fs'

const read = (name) => JSON.parse(readFileSync(new URL(`../src/mocks/${name}`, import.meta.url), 'utf8'))

const graph = read('graph.json')
const ask = read('ask.json')
const alerts = read('alerts.json')
const events = read('events.json')
const health = read('health.json')

const nodeUids = new Set(graph.nodes.map((n) => n.uid))
const edgeKeys = new Set(graph.edges.map((e) => `${e.type}|${e.src_uid}|${e.dst_uid}`))

let failures = 0
const check = (cond, msg) => {
  if (cond) console.log(`  ok  ${msg}`)
  else {
    failures += 1
    console.error(`  FAIL ${msg}`)
  }
}

console.log('ask envelope shape (API contract)')
for (const key of ['template', 'params', 'cypher', 'hops', 'run_time_ms', 'rows', 'path', 'answer']) {
  check(key in ask, `has "${key}"`)
}
check(ask.hops === 5, `hops = 5 (got ${ask.hops})`)
check(typeof ask.run_time_ms === 'number' && ask.run_time_ms > 0, `run_time_ms present (${ask.run_time_ms} ms)`)
check(ask.path.nodes.length === 9 && ask.path.edges.length === 8, 'path has 9 nodes / 8 edges (two Act 1 branches)')

console.log('ask path resolves against the graph fixture')
for (const n of ask.path.nodes) check(nodeUids.has(n.uid), `node in graph: ${n.uid}`)
for (const e of ask.path.edges) check(edgeKeys.has(`${e.type}|${e.src_uid}|${e.dst_uid}`), `edge in graph: ${e.type} ${e.src_uid} -> ${e.dst_uid}`)

console.log('Act 1 chain: Person->MAINTAINS->Repo->HAS_ISSUE->Issue->ABOUT->Topic<-COVERS<-Talk<-SPEAKS_AT<-Person')
const chain = [
  ['MAINTAINS', 'person:gh:manvendersingh21', 'repo:manvendersingh21/deepcore-n1'],
  ['HAS_ISSUE', 'repo:manvendersingh21/deepcore-n1', 'issue:manvendersingh21/deepcore-n1#1'],
  ['ABOUT', 'issue:manvendersingh21/deepcore-n1#1', 'topic:mcp-auth'],
  ['COVERS', 'talk:U26-09', 'topic:mcp-auth'],
  ['SPEAKS_AT', 'person:name:nick-taylor', 'talk:U26-09'],
]
for (const [type, src, dst] of chain) check(edgeKeys.has(`${type}|${src}|${dst}`), `${type} ${src} -> ${dst}`)

console.log('ask rows use the seeded data')
check(
  ask.rows.some((r) => r.talk && r.talk.startsWith("Open pull requests, don't merge them")),
  'a row returns talk:U26-09',
)
check(ask.rows.some((r) => r.speaker === 'Nick Taylor' && r.day === '2026-10-29' && r.start_time === '10:30'), 'row has Nick Taylor, day 2 10:30')
check(ask.rows.every((r) => r.issue && r.issue.includes('per-identity OAuth scopes')), 'rows cite the seeded issue')

console.log('alerts fixture')
for (const a of alerts) {
  check(typeof a.bridge_uid === 'string' && typeof a.draft === 'string' && typeof a.run_time_ms === 'number', `${a.bridge_uid} shape ok`)
  for (const n of a.path.nodes) check(nodeUids.has(n.uid), `alert path node in graph: ${n.uid}`)
  for (const e of a.path.edges) check(edgeKeys.has(`${e.type}|${e.src_uid}|${e.dst_uid}`), `alert path edge in graph: ${e.type}`)
}

console.log('events fixture')
let sorted = true
for (let i = 1; i < events.length; i++) if (events[i].ts < events[i - 1].ts) sorted = false
check(sorted, `oldest first (${events.length} events)`)

console.log('health counts match the graph fixture (per node label)')
const byLabel = {}
for (const n of graph.nodes) byLabel[n.label] = (byLabel[n.label] ?? 0) + 1
for (const [label, count] of Object.entries(byLabel)) {
  check(health.counts[label] === count, `${label}: health ${health.counts[label]} == graph ${count}`)
}

console.log(failures === 0 ? '\nALL FIXTURE CHECKS PASSED' : `\n${failures} CHECK(S) FAILED`)
process.exit(failures === 0 ? 0 : 1)
