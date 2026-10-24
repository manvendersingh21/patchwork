import { useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react'
import ForceGraph2D, {
  type ForceGraphMethods,
  type LinkObject,
  type NodeObject,
} from 'react-force-graph-2d'
import { LABEL_COLORS, degreesOf, graphEdgeKey, isInProgress, labelRgb, type Highlight } from '../lib/graph'
import type { GraphData, NodeLabel } from '../types'

type FGN = {
  uid: string
  label: string
  name: string
  state?: string
  degree: number
  x?: number
  y?: number
}

type FGL = {
  key: string
  type: string
  state?: string
  source: FGN
  target: FGN
}

function nodeRadius(n: FGN): number {
  return 3 + Math.sqrt(n.degree + 1) * 2.4
}

function truncate(s: string, max: number): string {
  return s.length > max ? `${s.slice(0, max - 1)}…` : s
}

export default function GraphPane({ graphData, highlight }: { graphData: GraphData; highlight: Highlight | null }) {
  const wrapRef = useRef<HTMLDivElement>(null)
  const fgRef = useRef<ForceGraphMethods<NodeObject<FGN>, LinkObject<FGN, FGL>> | undefined>(undefined)
  const nodesRef = useRef(new Map<string, FGN>())
  const sigRef = useRef('')
  const fittedRef = useRef(false)
  const [size, setSize] = useState({ w: 0, h: 0 })
  const [hoverUid, setHoverUid] = useState<string | null>(null)
  const [fgData, setFgData] = useState<{ nodes: FGN[]; links: FGL[] }>({ nodes: [], links: [] })

  useLayoutEffect(() => {
    const el = wrapRef.current
    if (!el) return
    const update = () => setSize({ w: el.clientWidth, h: el.clientHeight })
    update()
    const ro = new ResizeObserver(update)
    ro.observe(el)
    return () => ro.disconnect()
  }, [])

  // Merge the 2 s graph refresh in place by uid so the layout never jumps.
  useEffect(() => {
    const sig =
      graphData.nodes.map((n) => `${n.uid}:${n.name}:${n.state ?? ''}`).join('|') +
      '##' +
      graphData.edges.map((e) => `${graphEdgeKey(e)}:${e.state ?? ''}`).join('|')
    if (sig === sigRef.current) return
    sigRef.current = sig

    const deg = degreesOf(graphData.edges)
    const alive = new Set<string>()
    const nodes: FGN[] = []
    for (const gn of graphData.nodes) {
      alive.add(gn.uid)
      const d = deg.get(gn.uid) ?? 0
      let n = nodesRef.current.get(gn.uid)
      if (!n) {
        n = { uid: gn.uid, label: gn.label, name: gn.name, state: gn.state, degree: d }
        nodesRef.current.set(gn.uid, n)
      } else {
        n.name = gn.name
        n.label = gn.label
        n.state = gn.state
        n.degree = d
      }
      nodes.push(n)
    }
    for (const uid of [...nodesRef.current.keys()]) {
      if (!alive.has(uid)) nodesRef.current.delete(uid)
    }
    const links: FGL[] = []
    for (const e of graphData.edges) {
      const source = nodesRef.current.get(e.src_uid)
      const target = nodesRef.current.get(e.dst_uid)
      if (!source || !target) continue
      links.push({ key: graphEdgeKey(e), type: e.type, state: e.state, source, target })
    }
    setFgData({ nodes, links })
  }, [graphData])

  // Zoom to fit the highlighted nodes on every new highlight.
  useEffect(() => {
    if (!highlight) return
    const picked = [...nodesRef.current.values()].filter((n) => highlight.nodes.has(n.uid))
    if (picked.length === 0) return
    fgRef.current?.zoomToFit(700, 70, (n) => highlight.nodes.has(n.uid))
  }, [highlight])

  const paintNode = useCallback(
    (n: FGN, ctx: CanvasRenderingContext2D, globalScale: number) => {
      const dim = highlight != null && !highlight.nodes.has(n.uid)
      const r = nodeRadius(n)
      const x = n.x ?? 0
      const y = n.y ?? 0

      ctx.beginPath()
      ctx.arc(x, y, r, 0, 2 * Math.PI)
      ctx.fillStyle = `rgba(${labelRgb(n.label)},${dim ? 0.15 : 1})`
      ctx.fill()

      if (isInProgress(n.state)) {
        const phase = 0.5 + 0.5 * Math.sin(Date.now() / 280)
        ctx.beginPath()
        ctx.arc(x, y, r + 2 + 3 * phase, 0, 2 * Math.PI)
        ctx.strokeStyle = `rgba(251,191,36,${0.25 + 0.45 * phase})`
        ctx.lineWidth = 2 / globalScale
        ctx.stroke()
      }

      const showLabel = hoverUid === n.uid || (highlight?.nodes.has(n.uid) ?? false)
      if (showLabel) {
        const text = truncate(n.name, 26) + (isInProgress(n.state) ? ` · ${n.state}` : '')
        const fs = 16 / globalScale
        ctx.font = `${fs}px ui-sans-serif, system-ui, sans-serif`
        const tw = ctx.measureText(text).width
        ctx.fillStyle = 'rgba(2,6,23,0.85)'
        ctx.fillRect(x - tw / 2 - 4, y + r + 4, tw + 8, fs + 6)
        ctx.fillStyle = dim ? 'rgba(226,232,240,0.55)' : '#e2e8f0'
        ctx.textAlign = 'center'
        ctx.textBaseline = 'top'
        ctx.fillText(text, x, y + r + 7)
      }
    },
    [highlight, hoverUid],
  )

  const paintArea = useCallback((n: FGN, color: string, ctx: CanvasRenderingContext2D) => {
    ctx.beginPath()
    ctx.arc(n.x ?? 0, n.y ?? 0, nodeRadius(n) + 4, 0, 2 * Math.PI)
    ctx.fillStyle = color
    ctx.fill()
  }, [])

  const linkColor = useCallback(
    (l: FGL): string => {
      if (l.type === 'BRIDGE') {
        return l.state === 'DISCOVERED' ? 'rgba(251,191,36,0.9)' : 'rgba(74,222,128,0.95)'
      }
      if (highlight) {
        return highlight.edges.has(l.key) ? 'rgba(103,232,249,0.95)' : 'rgba(148,163,184,0.15)'
      }
      return 'rgba(148,163,184,0.45)'
    },
    [highlight],
  )

  const linkWidth = useCallback(
    (l: FGL): number => {
      if (highlight) return highlight.edges.has(l.key) ? 3.5 : l.type === 'BRIDGE' ? 1.5 : 0.75
      return l.type === 'BRIDGE' ? 2 : 1
    },
    [highlight],
  )

  const linkDash = useCallback(
    (l: FGL): number[] | null => (l.type === 'BRIDGE' && l.state === 'DISCOVERED' ? [6, 4] : null),
    [],
  )

  const particles = useCallback((l: FGL): number => (highlight?.edges.has(l.key) ? 4 : 0), [highlight])

  const onHover = useCallback((node: FGN | null) => {
    setHoverUid(node ? node.uid : null)
    document.body.style.cursor = node ? 'pointer' : 'default'
  }, [])

  return (
    <div className="graph-pane relative h-full w-full overflow-hidden bg-slate-950">
      <div ref={wrapRef} className="absolute inset-0">
        {size.w > 0 && size.h > 0 && (
          <ForceGraph2D<FGN, FGL>
            ref={fgRef}
            width={size.w}
            height={size.h}
            graphData={fgData}
            backgroundColor="rgba(2,6,23,0)"
            nodeId="uid"
            autoPauseRedraw={false}
            nodeRelSize={6}
            nodeLabel={() => ''}
            nodeCanvasObject={paintNode}
            nodePointerAreaPaint={paintArea}
            linkColor={linkColor}
            linkWidth={linkWidth}
            linkLineDash={linkDash}
            linkDirectionalParticles={particles}
            linkDirectionalParticleWidth={2.5}
            linkDirectionalParticleSpeed={0.006}
            linkDirectionalParticleColor={linkColor}
            onNodeHover={onHover}
            onEngineStop={() => {
              if (!fittedRef.current) {
                fittedRef.current = true
                fgRef.current?.zoomToFit(600, 70)
              }
            }}
          />
        )}
      </div>
      <div className="pointer-events-none absolute left-3 top-3 z-10 flex max-w-[calc(100%-1.5rem)] flex-wrap items-center gap-x-4 gap-y-1 rounded-md border border-emerald-900/60 bg-emerald-950/70 px-2.5 py-1.5 text-base text-emerald-100/90">
        {(Object.keys(LABEL_COLORS) as NodeLabel[]).map((l) => (
          <span key={l} className="flex items-center gap-1.5">
            <span className="h-2.5 w-2.5 rounded-full" style={{ background: LABEL_COLORS[l] }} />
            {l}
          </span>
        ))}
        <span className="flex items-center gap-1.5">
          <span className="h-0 w-4 border-t-2 border-dashed border-amber-400" />
          BRIDGE
        </span>
      </div>
      <div className="pointer-events-none absolute bottom-3 right-3 z-10 text-base text-emerald-200/40">
        Alt+I inject issue · Alt+R replay highlight
      </div>
    </div>
  )
}
