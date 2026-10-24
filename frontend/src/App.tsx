import { useCallback, useEffect, useRef, useState } from 'react'
import { ACT3_ISSUE, IS_MOCK, PRESENTER_HANDLE, api } from './api'
import { highlightFromPath, type Highlight } from './lib/graph'
import type { AgentEvent, Alert, AskEnvelope, GraphData, HealthStatus } from './types'
import AgentTicker from './components/AgentTicker'
import AlertCard from './components/AlertCard'
import EncounterBox from './components/EncounterBox'
import GraphPane from './components/GraphPane'
import { AnswerCard, default as AskPane } from './components/AskPane'
import DesktopIcon from './components/desktop/DesktopIcon'
import Marquee from './components/desktop/Marquee'
import Preloader from './components/desktop/Preloader'
import TopBar from './components/desktop/TopBar'
import Win, { type WinRect } from './components/desktop/Win'

const POLL_GRAPH_MS = 2000
const POLL_EVENTS_MS = 1000
const POLL_HEALTH_MS = 5000
const POLL_ALERTS_MS = 2000
const INJECT_MSG_MS = 5000

type WinId = 'graph' | 'ask' | 'notes' | 'sync' | 'agents'
type WinState = { open: boolean; minimized: boolean; z: number }

const WIN_TITLES: Record<WinId, string> = {
  graph: 'patchwork://graph',
  ask: 'patchwork://ask',
  notes: 'patchwork://notes',
  sync: 'patchwork://sync',
  agents: 'patchwork://agents',
}

/** Initial window geometry, responsive to the viewport at first paint. */
function initialRects(): Record<WinId, WinRect> {
  const vw = typeof window === 'undefined' ? 1440 : window.innerWidth
  const vh = typeof window === 'undefined' ? 900 : window.innerHeight
  if (vw < 900) {
    const full = { x: 8, y: 44, w: vw - 16, h: vh - 130 }
    return { graph: full, ask: full, notes: full, sync: full, agents: full }
  }
  return {
    graph: { x: Math.round(vw * 0.24), y: Math.round(vh * 0.08), w: Math.round(vw * 0.62), h: Math.round(vh * 0.7) },
    ask: { x: Math.round(vw * 0.05), y: Math.round(vh * 0.1), w: 480, h: Math.round(vh * 0.52) },
    notes: { x: Math.round(vw * 0.05), y: Math.round(vh * 0.5), w: 460, h: Math.round(vh * 0.3) },
    sync: { x: Math.round(vw * 0.07), y: Math.round(vh * 0.34), w: 440, h: 190 },
    agents: { x: Math.round(vw * 0.12), y: Math.round(vh * 0.6), w: 500, h: Math.round(vh * 0.28) },
  }
}

// -- tiny inline glyphs for the desktop icons --------------------------------

const GlyphGraph = () => (
  <svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4">
    <circle cx="5" cy="18" r="2" />
    <circle cx="12" cy="5" r="2" />
    <circle cx="19" cy="15" r="2" />
    <circle cx="7" cy="9" r="1.4" />
    <path d="M6.2 10.1 11 6.7M13.8 6.3 17.6 13.4M7 16.2 5.8 16.8M8.6 8.2l8.4 6.2" />
  </svg>
)
const GlyphAsk = () => (
  <svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4">
    <path d="M4 5h16v11H9l-5 4V5Z" />
    <path d="M8 9h8M8 12h5" />
  </svg>
)
const GlyphNotes = () => (
  <svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4">
    <path d="M5 4h14v16H5z" />
    <path d="M8 8h8M8 12h8M8 16h4" />
  </svg>
)
const GlyphSync = () => (
  <svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4">
    <path d="M20 12a8 8 0 1 1-2.3-5.6" />
    <path d="M20 3v4h-4" />
  </svg>
)
const GlyphAgents = () => (
  <svg width="34" height="34" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.4">
    <circle cx="12" cy="12" r="2.4" />
    <circle cx="5" cy="5" r="1.6" />
    <circle cx="19" cy="5" r="1.6" />
    <circle cx="5" cy="19" r="1.6" />
    <circle cx="19" cy="19" r="1.6" />
    <path d="M6.3 6.3 10.2 10.2M17.7 6.3 13.8 10.2M6.3 17.7 10.2 13.8M17.7 17.7 13.8 13.8" />
  </svg>
)

// -- sync window body ---------------------------------------------------------

function SyncBox() {
  const [handle, setHandle] = useState(PRESENTER_HANDLE)
  const [status, setStatus] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  const submit = async () => {
    if (!handle.trim() || busy) return
    setBusy(true)
    try {
      const res = await api.sync(handle.trim())
      setStatus(`✓ sync requested for ${res.person_uid}`)
    } catch {
      setStatus('✗ sync failed')
    } finally {
      setBusy(false)
    }
  }

  return (
    <section className="space-y-2 p-3">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          void submit()
        }}
        className="flex items-center gap-2"
      >
        <span className="shrink-0 font-mono text-base text-emerald-200/40">github.com/</span>
        <input
          value={handle}
          onChange={(e) => setHandle(e.target.value)}
          className="min-w-0 flex-1 rounded border border-emerald-900/70 bg-emerald-950/50 px-2 py-1 text-base text-slate-50 focus:border-emerald-400 focus:outline-none"
        />
        <button
          type="submit"
          disabled={busy || !handle.trim()}
          className="shrink-0 rounded border border-accent/60 px-2.5 py-1 font-mono text-base text-accent hover:bg-accent/10 disabled:opacity-40"
        >
          {busy ? 'Syncing…' : 'Sync my codebase'}
        </button>
      </form>
      {status && <p className="truncate font-mono text-base text-emerald-100/60">{status}</p>}
    </section>
  )
}

// -- app ----------------------------------------------------------------------

export default function App() {
  const [health, setHealth] = useState<HealthStatus | null>(null)
  const [graph, setGraph] = useState<GraphData>({ nodes: [], edges: [] })
  const [events, setEvents] = useState<AgentEvent[]>([])
  const [answers, setAnswers] = useState<AskEnvelope[]>([])
  const [alert, setAlert] = useState<Alert | null>(null)
  const [highlight, setHighlight] = useState<Highlight | null>(null)
  const [busyAsk, setBusyAsk] = useState(false)
  const [injectMsg, setInjectMsg] = useState<string | null>(null)
  const [booted, setBooted] = useState(false)

  const [wins, setWins] = useState<Record<WinId, WinState>>({
    graph: { open: false, minimized: false, z: 0 },
    ask: { open: false, minimized: false, z: 0 },
    notes: { open: false, minimized: false, z: 0 },
    sync: { open: false, minimized: false, z: 0 },
    agents: { open: false, minimized: false, z: 0 },
  })
  const [rects] = useState(initialRects)
  const zRef = useRef(10)

  const nonceRef = useRef(0)
  const sinceRef = useRef(0)
  const lastAnswerRef = useRef<AskEnvelope | null>(null)
  const alertRef = useRef<Alert | null>(null)
  const askingRef = useRef(false)

  const nextNonce = () => {
    nonceRef.current += 1
    return nonceRef.current
  }

  const focusWin = useCallback((id: WinId) => {
    zRef.current += 1
    const z = zRef.current
    setWins((w) => ({ ...w, [id]: { ...w[id], z } }))
  }, [])

  const openWin = useCallback(
    (id: WinId) => {
      setWins((w) => (w[id].open && !w[id].minimized ? w : { ...w, [id]: { open: true, minimized: false, z: ++zRef.current } }))
    },
    [],
  )
  const closeWin = useCallback((id: WinId) => setWins((w) => ({ ...w, [id]: { ...w[id], open: false } })), [])
  const minimizeWin = useCallback(
    (id: WinId) => setWins((w) => ({ ...w, [id]: { ...w[id], minimized: true } })),
    [],
  )

  // After the preloader: open the hero windows with a slight cascade.
  const onBooted = useCallback(() => {
    setBooted(true)
    openWin('graph')
    setTimeout(() => openWin('ask'), 260)
  }, [openWin])

  // /api/graph every 2 s — GraphPane merges nodes in place by uid.
  useEffect(() => {
    let alive = true
    const load = async () => {
      try {
        const g = await api.graph()
        if (alive) setGraph(g)
      } catch {
        /* backend offline; keep last graph */
      }
    }
    void load()
    const id = setInterval(load, POLL_GRAPH_MS)
    return () => {
      alive = false
      clearInterval(id)
    }
  }, [])

  // /api/events every 1 s.
  useEffect(() => {
    let alive = true
    const load = async () => {
      try {
        const fresh = await api.events(sinceRef.current)
        if (!alive || fresh.length === 0) return
        sinceRef.current = fresh[fresh.length - 1].ts
        setEvents((prev) => [...prev, ...fresh].slice(-40))
      } catch {
        /* ignore */
      }
    }
    void load()
    const id = setInterval(load, POLL_EVENTS_MS)
    return () => {
      alive = false
      clearInterval(id)
    }
  }, [])

  useEffect(() => {
    let alive = true
    const load = async () => {
      try {
        const h = await api.health()
        if (alive) setHealth(h)
      } catch {
        /* ignore */
      }
    }
    void load()
    const id = setInterval(load, POLL_HEALTH_MS)
    return () => {
      alive = false
      clearInterval(id)
    }
  }, [])

  // /api/alerts — a new ACTION_READY bridge pops a toast and raises the graph.
  useEffect(() => {
    let alive = true
    const load = async () => {
      try {
        const list = await api.alerts()
        if (!alive || list.length === 0 || alertRef.current) return
        const a = list[0]
        alertRef.current = a
        setAlert(a)
        setHighlight(highlightFromPath(a.path, nextNonce()))
        openWin('graph')
      } catch {
        /* ignore */
      }
    }
    void load()
    const id = setInterval(load, POLL_ALERTS_MS)
    return () => {
      alive = false
      clearInterval(id)
    }
  }, [openWin])

  const ask = useCallback(
    async (question: string) => {
      if (askingRef.current) return
      askingRef.current = true
      setBusyAsk(true)
      try {
        const env = await api.ask(question)
        lastAnswerRef.current = env
        setAnswers((prev) => [env, ...prev])
        setHighlight(highlightFromPath(env.path, nextNonce()))
        openWin('graph')
      } catch (err) {
        console.error('ask failed', err)
      } finally {
        askingRef.current = false
        setBusyAsk(false)
      }
    },
    [openWin],
  )

  const dismissAlert = useCallback(() => {
    const a = alertRef.current
    alertRef.current = null
    setAlert(null)
    if (a) void api.markAlertSeen(a.bridge_uid).catch(() => undefined)
    const last = lastAnswerRef.current
    setHighlight(last ? highlightFromPath(last.path, nextNonce()) : null)
  }, [])

  const injectIssue = useCallback(async () => {
    try {
      const res = await api.injectIssue(ACT3_ISSUE.title, ACT3_ISSUE.body)
      setInjectMsg(`injected ${res.issue_uid} — normal pipeline running`)
    } catch (err) {
      console.error('inject failed', err)
      setInjectMsg('inject failed')
    }
    setTimeout(() => setInjectMsg(null), INJECT_MSG_MS)
  }, [])

  const replayHighlight = useCallback(() => {
    const last = lastAnswerRef.current
    if (last) setHighlight(highlightFromPath(last.path, nextNonce()))
  }, [])

  // Fallback hotkeys — active only when no input has focus.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!e.altKey || e.ctrlKey || e.metaKey || e.shiftKey) return
      const el = document.activeElement
      const tag = el instanceof HTMLElement ? el.tagName : ''
      if (tag === 'INPUT' || tag === 'TEXTAREA' || (el instanceof HTMLElement && el.isContentEditable)) return
      if (e.code === 'KeyI') {
        e.preventDefault()
        void injectIssue()
      } else if (e.code === 'KeyR') {
        e.preventDefault()
        replayHighlight()
      }
    }
    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [injectIssue, replayHighlight])

  const topZ = Math.max(...Object.values(wins).map((w) => w.z))

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden bg-desktop font-sans text-slate-900">
      {!booted && <Preloader onDone={onBooted} />}
      <TopBar health={health} mock={IS_MOCK} />

      <div className="desktop-wallpaper relative flex min-h-0 flex-1">
        {/* desktop icons */}
        <div className="absolute left-2 top-3 z-[2] flex flex-col gap-2">
          <DesktopIcon label="Graph" onOpen={() => openWin('graph')}>
            <GlyphGraph />
          </DesktopIcon>
          <DesktopIcon label="Ask" onOpen={() => openWin('ask')}>
            <GlyphAsk />
          </DesktopIcon>
          <DesktopIcon label="Notes" onOpen={() => openWin('notes')}>
            <GlyphNotes />
          </DesktopIcon>
          <DesktopIcon label="Sync" onOpen={() => openWin('sync')}>
            <GlyphSync />
          </DesktopIcon>
          <DesktopIcon label="Agents" onOpen={() => openWin('agents')}>
            <GlyphAgents />
          </DesktopIcon>
        </div>

        {/* windows */}
        {(Object.keys(WIN_TITLES) as WinId[]).map((id) => (
          <Win
            key={id}
            title={WIN_TITLES[id]}
            open={wins[id].open}
            minimized={wins[id].minimized}
            focused={wins[id].z === topZ}
            rect={rects[id]}
            onPointerDown={() => focusWin(id)}
            onClose={() => closeWin(id)}
            onMinimize={() => minimizeWin(id)}
          >
            {id === 'graph' && <GraphPane graphData={graph} highlight={highlight} />}
            {id === 'ask' && (
              <div className="flex h-full flex-col">
                <div className="shrink-0 border-b border-slate-800/70 p-3">
                  <AskPane onAsk={(q) => void ask(q)} busy={busyAsk} />
                </div>
                <section className="min-h-0 flex-1 space-y-3 overflow-y-auto p-3" aria-label="answers">
                  {answers.length === 0 && (
                    <p className="text-base text-slate-500">
                      Ask a question — every answer comes back with its path, the Cypher and FalkorDB&rsquo;s
                      own execution time.
                    </p>
                  )}
                  {answers.map((env, i) => (
                    <AnswerCard key={`${nonceRef.current}-${i}-${env.template}`} env={env} />
                  ))}
                </section>
              </div>
            )}
            {id === 'notes' && (
              <div className="h-full overflow-y-auto">
                <EncounterBox />
              </div>
            )}
            {id === 'sync' && <SyncBox />}
            {id === 'agents' && (
              <div className="h-full overflow-y-auto">
                <AgentTicker events={events} />
              </div>
            )}
          </Win>
        ))}

        {/* toasts */}
        <div className="pointer-events-none absolute right-3 top-3 z-[600] flex w-[26rem] max-w-[calc(100vw-1.5rem)] flex-col items-end gap-2">
          {injectMsg && (
            <div className="toast-in pointer-events-auto rounded-md border border-amber-400/40 bg-slate-900/95 px-3 py-1.5 font-mono text-base text-amber-300 shadow-lg">
              {injectMsg}
            </div>
          )}
          {alert && (
            <div className="pointer-events-auto">
              <AlertCard alert={alert} onDismiss={dismissAlert} />
            </div>
          )}
        </div>
      </div>

      <Marquee events={events} />
    </div>
  )
}
