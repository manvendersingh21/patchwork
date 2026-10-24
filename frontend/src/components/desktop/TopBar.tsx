import { useEffect, useState } from 'react'
import { PRESENTER_HANDLE } from '../../api'
import type { HealthStatus } from '../../types'

function HealthDot({ name, ok }: { name: string; ok: boolean }) {
  return (
    <span className="flex items-center gap-1">
      <span className={`h-1.5 w-1.5 rounded-full ${ok ? 'bg-emerald-400' : 'bg-rose-500'}`} />
      <span className="font-mono text-base text-slate-400">{name}</span>
    </span>
  )
}

/**
 * Slim desktop-style top bar: wordmark, handle, health, node count, clock.
 */
export default function TopBar({ health, mock }: { health: HealthStatus | null; mock: boolean }) {
  const [clock, setClock] = useState(() => new Date())
  useEffect(() => {
    const id = setInterval(() => setClock(new Date()), 1000)
    return () => clearInterval(id)
  }, [])

  const nodeCount = health
    ? Object.entries(health.counts)
        .filter(([label]) => label !== 'AgentEvent' && label !== 'BRIDGE')
        .reduce((sum, [, count]) => sum + count, 0)
    : 0

  return (
    <header className="relative z-[500] flex h-8 shrink-0 items-center gap-3 border-b border-slate-800/80 bg-slate-950/70 px-3 backdrop-blur">
      <span className="font-mono text-lg font-bold tracking-tight text-slate-100">
        Patch<span className="text-accent">Work</span>
      </span>
      {mock && (
        <span className="rounded bg-amber-400/15 px-1.5 py-0.5 font-mono text-base text-amber-300">mock</span>
      )}
      <span className="truncate font-mono text-base text-slate-500">@{PRESENTER_HANDLE}</span>
      <div className="ml-auto flex items-center gap-3">
        {health ? (
          <>
            <HealthDot name="falkordb" ok={health.falkordb === true} />
            <HealthDot name="openai" ok={health.openai === true} />
            <HealthDot name="github" ok={health.github === true} />
            <span className="font-mono text-base text-slate-500">{nodeCount} nodes</span>
          </>
        ) : (
          <span className="font-mono text-base text-slate-500">checking health…</span>
        )}
        <span className="font-mono text-base tabular-nums text-slate-400">
          {clock.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
        </span>
      </div>
    </header>
  )
}
