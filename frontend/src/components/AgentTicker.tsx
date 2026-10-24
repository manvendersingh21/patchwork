import { agentColor, fmtClock } from '../lib/graph'
import type { AgentEvent } from '../types'

export default function AgentTicker({ events }: { events: AgentEvent[] }) {
  const last = events.slice(-8)
  return (
    <div className="shrink-0 border-t border-slate-800 bg-slate-900/60 px-4 py-1.5">
      <div className="flex items-center justify-between">
        <h2 className="text-base font-semibold text-slate-300">Agents</h2>
        <span className="text-base text-slate-500">live</span>
      </div>
      <ul className="mt-0.5 space-y-0.5">
        {last.length === 0 && <li className="text-base text-slate-500">waiting for agent events…</li>}
        {last.map((e) => (
          <li key={e.uid} className="flex items-baseline gap-1.5 text-base leading-tight">
            <span
              className="h-2 w-2 shrink-0 self-center rounded-full"
              style={{ background: agentColor(e.agent) }}
            />
            <span className="font-medium" style={{ color: agentColor(e.agent) }}>
              {e.agent}
            </span>
            <span className="text-slate-500">·</span>
            <span className="max-w-[9rem] truncate text-slate-300">{e.target}</span>
            <span className="text-slate-500">·</span>
            <span className="truncate text-slate-400">{e.detail}</span>
            <span className="ml-auto shrink-0 font-mono text-slate-600">{fmtClock(e.ts)}</span>
          </li>
        ))}
      </ul>
    </div>
  )
}
