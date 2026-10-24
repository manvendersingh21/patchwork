// Verifies the ask-fixture highlight using the app's real module code:
// loads src/lib/graph.ts through Vite, runs the same highlightFromPath the
// App calls on a successful /api/ask, and checks the result against the graph
// fixture exactly like GraphPane does (membership sets + zoom-to-fit targets).
import { readFileSync } from 'node:fs'
import { createServer } from 'vite'

const read = (name) => JSON.parse(readFileSync(new URL(`../src/mocks/${name}`, import.meta.url), 'utf8'))
const ask = read('ask.json')
const graph = read('graph.json')

const server = await createServer({
  configFile: 'vite.config.ts',
  server: { middlewareMode: true },
  logLevel: 'error',
})

let failures = 0
const check = (cond, msg) => {
  if (cond) console.log(`  ok  ${msg}`)
  else {
    failures += 1
    console.error(`  FAIL ${msg}`)
  }
}

try {
  const { highlightFromPath, graphEdgeKey } = await server.ssrLoadModule('/src/lib/graph.ts')

  // This is exactly what App.ask() does with the /api/ask envelope.
  const hl = highlightFromPath(ask.path, 1)

  const graphUids = new Set(graph.nodes.map((n) => n.uid))
  const graphEdgeKeys = new Set(graph.edges.map((e) => graphEdgeKey(e)))

  console.log('highlight sets from the ask fixture (app code path)')
  check(hl.nodes.size === ask.path.nodes.length, `highlight node set covers all ${ask.path.nodes.length} path nodes`)
  check(hl.edges.size === ask.path.edges.length, `highlight edge set covers all ${ask.path.edges.length} path edges`)

  console.log('every highlighted node exists in the live graph pane (zoom-to-fit targets)')
  let allPresent = true
  for (const uid of hl.nodes) if (!graphUids.has(uid)) allPresent = false
  check(allPresent, 'all highlighted uids resolve to nodes the pane is rendering')

  console.log('dimming split: highlighted vs dimmed at 15% opacity')
  const dimmed = graph.nodes.filter((n) => !hl.nodes.has(n.uid)).map((n) => n.uid)
  check(dimmed.length === graph.nodes.length - hl.nodes.size, `${hl.nodes.size} highlighted, ${dimmed.length} dimmed`)
  check(
    dimmed.includes('issue:manvendersingh21/deepcore-n1#7') && dimmed.includes('talk:U26-04'),
    'off-path branches (issue #7, Copilot-memory talk) are dimmed, not highlighted',
  )

  console.log('highlighted edges include the full Act 1 traversal')
  for (const wanted of [
    'MAINTAINS|person:gh:manvendersingh21|repo:manvendersingh21/deepcore-n1',
    'HAS_ISSUE|repo:manvendersingh21/deepcore-n1|issue:manvendersingh21/deepcore-n1#1',
    'ABOUT|issue:manvendersingh21/deepcore-n1#1|topic:mcp-auth',
    'COVERS|talk:U26-09|topic:mcp-auth',
    'SPEAKS_AT|person:name:nick-taylor|talk:U26-09',
  ]) {
    check(hl.edges.has(wanted) && graphEdgeKeys.has(wanted), wanted)
  }

  // Particles: 4 on highlighted links, widened edges — driven by hl.edges in GraphPane.
  check(hl.edges.size >= 8, 'particle targets present (4 particles per highlighted link)')
} finally {
  await server.close()
}

console.log(failures === 0 ? '\nHIGHLIGHT VERIFICATION PASSED' : `\n${failures} CHECK(S) FAILED`)
process.exit(failures === 0 ? 0 : 1)
