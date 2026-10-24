import { useState } from 'react'
import type { AskEnvelope } from '../types'

export default function GraphProofDrawer({ env }: { env: AskEnvelope }) {
  const [open, setOpen] = useState(false)
  return (
    <div>
      <button
        onClick={() => setOpen((o) => !o)}
        className="rounded border border-slate-700 px-2 py-0.5 text-base text-slate-300 hover:bg-slate-800"
      >
        {open ? '▾' : '▸'} Graph proof
      </button>
      {open && (
        <div className="mt-2 space-y-1.5 rounded border border-slate-700 bg-slate-950/70 p-2 text-base">
          <div>
            <span className="text-slate-500">template </span>
            <span className="font-mono text-sky-300">{env.template}</span>
          </div>
          <div className="break-all">
            <span className="text-slate-500">params </span>
            <span className="font-mono text-slate-300">{JSON.stringify(env.params)}</span>
          </div>
          <div className="flex flex-wrap gap-x-4">
            <span>
              <span className="text-slate-500">hops </span>
              <span className="font-mono text-slate-200">{env.hops}</span>
            </span>
            <span>
              <span className="text-slate-500">FalkorDB </span>
              <span className="font-mono text-emerald-300">{env.run_time_ms} ms</span>
            </span>
            <span>
              <span className="text-slate-500">rows </span>
              <span className="font-mono text-slate-200">{env.rows.length}</span>
            </span>
          </div>
          <pre className="max-h-44 overflow-auto whitespace-pre-wrap break-all rounded bg-black/40 p-2 font-mono text-base text-slate-300">
            {env.cypher}
          </pre>
        </div>
      )}
    </div>
  )
}
