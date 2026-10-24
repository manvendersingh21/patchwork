import { useCallback, useEffect, useRef, useState } from 'react'
import { ACT3_ISSUE, IS_MOCK, PRESENTER_HANDLE, api } from './api'
import { highlightFromPath, type Highlight } from './lib/graph'
import type { AgentEvent, Alert, AskEnvelope, GraphData, HealthStatus } from './types'
import AgentTicker from './components/AgentTicker'
import AlertCard from './components/AlertCard'
import EncounterBox from './components/EncounterBox'
import GraphPane from './components/GraphPane'
import { AnswerCard, default as AskPane } from './components/AskPane'

const POLL_GRAPH_MS = 2000
const POLL_EVENTS_MS = 1000
const POLL_HEALTH_MS = 5000
const POLL_ALERTS_MS = 2000
const INJECT_MSG_MS = 5000

function HealthDot({ name, ok }: { name: string; ok: boolean }) {
  return (
    <span className="flex items-center gap-1.5">
      <span className={`h-2.5 w-2.5 rounded-full ${ok ? 'bg-emerald-400' : 'bg-rose-500'}`} />
      <span className="text-slate-400">{name}</span>
    </span>
  )
}

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
    <section className="space-y-1">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          void submit()
        }}
        className="flex items-center gap-2"
      >
        <span className="shrink-0 font-mono text-base text-slate-500">github.com/</span>
        <input
          value={handle}
          onChange={(e) => setHandle(e.target.value)}
          className="min-w-0 flex-1 rounded border border-slate-700 bg-slate-900/70 px-2 py-1 text-base text-slate-100 focus:border-sky-500 focus:outline-none"
        />
        <button
          type="submit"
          disabled={busy || !handle.trim()}
          className="shrink-0 rounded border border-violet-400/50 px-2.5 py-1 text-base text-violet-300 hover:bg-violet-400/10 disabled:opacity-40"
        >
          {busy ? 'Syncing…' : 'Sync my codebase'}
        </button>
      </form>
      {status && <p className="truncate text-base text-slate-400">{status}</p>}
    </section>
  )
}

export default function App() {
  const [health, setHealth] = useState<HealthStatus | null>(null)
  const [graph, setGraph] = useState<GraphData>({ nodes: [], edges: [] })
  const [events, setEvents] = useState<AgentEvent[]>([])
  const [answers, setAnswers] = useState<AskEnvelope[]>([])
  const [alert, setAlert] = useState<Alert | null>(null)
  const [highlight, setHighlight] = useState<Highlight | null>(null)
  const [busyAsk, setBusyAsk] = useState(false)
  const [injectMsg, setInjectMsg] = useState<string | null>(null)

  const nonceRef = useRef(0)
  const sinceRef = useRef(0)
  const lastAnswerRef = useRef<AskEnvelope | null>(null)
  const alertRef = useRef<Alert | null>(null)
  const askingRef = useRef(false)

  const nextNonce = () => {
    nonceRef.current += 1
    return nonceRef.current
  }

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

  // /api/events every 1 s — oldest first, ticker keeps the last 8.
  useEffect(() => {
    let alive = true
    const load = async () => {
      try {
        const fresh = await api.events(sinceRef.current)
        if (!alive || fresh.length === 0) return
        sinceRef.current = fresh[fresh.length - 1].ts
        setEvents((prev) => [...prev, ...fresh].slice(-8))
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

  // /api/alerts — a new ACTION_READY bridge slides in and takes the highlight.
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
  }, [])

  const ask = useCallback(async (question: string) => {
    if (askingRef.current) return
    askingRef.current = true
    setBusyAsk(true)
    try {
      const env = await api.ask(question)
      lastAnswerRef.current = env
      setAnswers((prev) => [env, ...prev])
      setHighlight(highlightFromPath(env.path, nextNonce()))
    } catch (err) {
      console.error('ask failed', err)
    } finally {
      askingRef.current = false
      setBusyAsk(false)
    }
  }, [])

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

  const nodeCount = health
    ? Object.entries(health.counts)
        .filter(([label]) => label !== 'AgentEvent' && label !== 'BRIDGE')
        .reduce((sum, [, count]) => sum + count, 0)
    : 0

  return (
    <div className="flex h-screen w-screen overflow-hidden bg-slate-950 text-slate-100">
      <aside className="flex h-full w-2/5 min-w-0 flex-col border-r border-slate-800">
        <header className="flex shrink-0 items-center gap-3 border-b border-slate-800 px-4 py-2">
          <h1 className="text-xl font-bold tracking-tight">PatchWork</h1>
          {IS_MOCK && (
            <span className="rounded bg-amber-400/15 px-1.5 py-0.5 text-base text-amber-300">mock</span>
          )}
          <span className="truncate font-mono text-base text-slate-400">@{PRESENTER_HANDLE}</span>
          <div className="ml-auto flex shrink-0 items-center gap-3 text-base">
            {health ? (
              <>
                <HealthDot name="FalkorDB" ok={health.falkordb === 'ok'} />
                <HealthDot name="OpenAI" ok={health.openai === 'ok'} />
                <HealthDot name="GitHub" ok={health.github === 'ok'} />
                <span className="text-slate-500">{nodeCount} nodes</span>
              </>
            ) : (
              <span className="text-slate-500">checking health…</span>
            )}
          </div>
        </header>

        <div className="flex shrink-0 flex-col gap-3 px-4 pt-3 pb-2">
          <SyncBox />
          <AskPane onAsk={(q) => void ask(q)} busy={busyAsk} />
          <EncounterBox />
        </div>

        <section className="min-h-0 flex-1 space-y-3 overflow-y-auto px-4 pb-3" aria-label="answers">
          {answers.length === 0 && (
            <p className="text-base text-slate-500">
              Ask a question — every answer comes back with its path, the Cypher and FalkorDB&rsquo;s own
              execution time.
            </p>
          )}
          {answers.map((env, i) => (
            <AnswerCard key={`${nonceRef.current}-${i}-${env.template}`} env={env} />
          ))}
        </section>

        {injectMsg && (
          <p className="shrink-0 border-t border-slate-800 bg-amber-400/10 px-4 py-1 text-base text-amber-300">
            {injectMsg}
          </p>
        )}
        <AgentTicker events={events} />
      </aside>

      <main className="relative h-full w-3/5 min-w-0">
        <GraphPane graphData={graph} highlight={highlight} />
        {alert && <AlertCard alert={alert} onDismiss={dismissAlert} />}
      </main>
    </div>
  )
}
