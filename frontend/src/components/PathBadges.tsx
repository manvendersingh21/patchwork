import { labelColor, pathOrder } from '../lib/graph'
import type { PathShape } from '../types'

export default function PathBadges({ path }: { path: PathShape }) {
  const items = pathOrder(path)
  return (
    <div className="flex flex-wrap items-center gap-x-1 gap-y-1">
      {items.map((it, i) => (
        <span key={it.node.uid} className="flex items-center gap-1">
          {i > 0 && <span className="text-slate-600">›</span>}
          <span className="group relative">
            <span
              className="inline-block max-w-[15rem] truncate rounded border px-1.5 py-0.5 text-base"
              style={{ borderColor: `${labelColor(it.node.label)}66`, color: labelColor(it.node.label) }}
            >
              {it.node.name}
            </span>
            {it.via && (
              <span className="pointer-events-none absolute left-0 top-full z-30 mt-1 hidden w-max max-w-[20rem] rounded border border-slate-700 bg-slate-900 px-2 py-1 text-left text-base text-slate-300 shadow-lg group-hover:block">
                <span className="font-mono text-slate-400">
                  {it.via.type}
                  {typeof it.via.confidence === 'number' ? ` · ${it.via.confidence}` : ''}
                </span>
                {it.via.evidence && <span className="block">&ldquo;{it.via.evidence}&rdquo;</span>}
              </span>
            )}
          </span>
        </span>
      ))}
    </div>
  )
}
