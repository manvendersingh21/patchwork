import { agentColor, fmtClock } from '../../lib/graph'
import type { AgentEvent } from '../../types'

/**
 * Bottom marquee strip: the live agent events scroll across, reference-style.
 * Hover pauses it. With no events yet it shows a standby line.
 */
export default function Marquee({ events }: { events: AgentEvent[] }) {
  const recent = events.slice(-10)
  const items =
    recent.length > 0
      ? recent.map((e) => ({
          key: e.uid,
          left: `${e.agent} · ${e.target}`,
          right: `${e.detail} · ${fmtClock(e.ts)}`,
          color: agentColor(e.agent),
        }))
      : [
          { key: 'standby', left: 'PATCHWORK', right: 'waiting for agent events…', color: '#64748b' },
        ]

  return (
    <footer className="relative z-[500] shrink-0 overflow-hidden border-t border-slate-800/80 bg-slate-950/70 py-1 backdrop-blur">
      <div className="marquee-track font-mono text-base">
        {[0, 1].map((copy) => (
          <span key={copy} className="flex">
            {items.map((it) => (
              <span key={`${copy}-${it.key}`} className="mx-6 flex items-center gap-2">
                <span className="h-1.5 w-1.5 rounded-full" style={{ background: it.color }} />
                <span style={{ color: it.color }}>{it.left}</span>
                <span className="text-slate-500">{it.right}</span>
              </span>
            ))}
          </span>
        ))}
      </div>
    </footer>
  )
}
