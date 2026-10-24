import { useState } from 'react'
import GraphProofDrawer from './GraphProofDrawer'
import PathBadges from './PathBadges'
import type { AskEnvelope } from '../types'

export const CHIPS = [
  'Which talks after lunch on day 1 help with my open issues?',
  'Which talks on day 2 morning help with my open issues?',
  'Who should I meet that I haven\u2019t yet?',
]

export default function AskPane({ onAsk, busy }: { onAsk: (question: string) => void; busy: boolean }) {
  const [question, setQuestion] = useState('')

  const submit = () => {
    if (!question.trim() || busy) return
    onAsk(question.trim())
    setQuestion('')
  }

  return (
    <section className="space-y-2">
      <form
        onSubmit={(e) => {
          e.preventDefault()
          submit()
        }}
        className="flex gap-2"
      >
        <input
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          placeholder="Ask the graph…"
          className="min-w-0 flex-1 rounded border border-emerald-900/70 bg-emerald-950/50 px-2 py-1.5 text-base text-slate-50 placeholder:text-emerald-200/60 focus:border-emerald-400 focus:outline-none"
        />
        <button
          type="submit"
          disabled={busy || !question.trim()}
          className="shrink-0 rounded border border-emerald-400/60 px-2.5 py-1.5 text-base text-emerald-200 hover:bg-emerald-400/10 disabled:opacity-40"
        >
          {busy ? 'Asking…' : 'Ask'}
        </button>
      </form>
      <div className="flex flex-wrap gap-1.5">
        {CHIPS.map((chip) => (
          <button
            key={chip}
            onClick={() => onAsk(chip)}
            disabled={busy}
            className="rounded-full border border-emerald-900/60 bg-emerald-950/40 px-2.5 py-1 text-base text-emerald-100/80 hover:border-emerald-400/60 hover:text-emerald-100 disabled:opacity-40"
          >
            {chip}
          </button>
        ))}
      </div>
    </section>
  )
}

export function AnswerCard({ env }: { env: AskEnvelope }) {
  return (
    <article className="rounded-lg border border-emerald-900/50 bg-emerald-950/30 p-3">
      <p className="text-base text-slate-100">{env.answer}</p>
      <div className="mt-2">
        <PathBadges path={env.path} />
      </div>
      <div className="mt-2 flex flex-wrap items-center gap-2 text-base">
        <span className="rounded bg-emerald-900/60 px-1.5 py-0.5 font-mono text-emerald-200">{env.template}</span>
        <span className="text-emerald-100/60">{env.hops} hops</span>
        <span className="text-emerald-200/40">·</span>
        <span className="text-emerald-100/60">
          FalkorDB <span className="font-mono text-emerald-300">{env.run_time_ms} ms</span>
        </span>
        <span className="ml-auto">
          <GraphProofDrawer env={env} />
        </span>
      </div>
    </article>
  )
}
