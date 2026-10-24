import { useState } from 'react'
import type { AskEnvelope } from '../types'

export default function GraphProofDrawer({ env }: { env: AskEnvelope }) {
  const [open, setOpen] = useState(false)
  return (
    <div>
      <button
        onClick={() => setOpen((o) => !o)}
        className="rounded border border-emerald-400/60 px-2 py-0.5 text-base text-emerald-200 hover:bg-emerald-400/10"
      >
        {open ? '▾' : '▸'} Graph proof
      </button>
      {open && (
        <div className="mt-2 space-y-1.5 rounded border border-emerald-900/60 bg-[#04140d]/70 p-2 text-base">
          <div>
            <span className="text-emerald-100/70">template </span>
            <span className="font-mono text-emerald-300">{env.template}</span>
          </div>
          <div className="break-all">
            <span className="text-emerald-100/70">params </span>
            <span className="font-mono text-emerald-100/60">{JSON.stringify(env.params)}</span>
          </div>
          <div className="flex flex-wrap gap-x-4">
            <span>
              <span className="text-emerald-100/70">hops </span>
              <span className="font-mono text-slate-100">{env.hops}</span>
            </span>
            <span>
              <span className="text-emerald-100/70">FalkorDB </span>
              <span className="font-mono text-emerald-300">{env.run_time_ms} ms</span>
            </span>
            <span>
              <span className="text-emerald-100/70">rows </span>
              <span className="font-mono text-slate-100">{env.rows.length}</span>
            </span>
          </div>
          <pre className="max-h-44 overflow-auto whitespace-pre-wrap break-all rounded bg-black/40 p-2 font-mono text-base text-emerald-100/60">
            {env.cypher}
          </pre>
        </div>
      )}
    </div>
  )
}
