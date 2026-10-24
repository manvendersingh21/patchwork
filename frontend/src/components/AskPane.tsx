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
          className="min-w-0 flex-1 rounded border border-slate-700 bg-slate-900/70 px-2 py-1.5 text-base text-slate-100 placeholder:text-slate-600 focus:border-sky-500 focus:outline-none"
        />
        <button
          type="submit"
          disabled={busy || !question.trim()}
          className="shrink-0 rounded border border-sky-400/50 px-2.5 py-1.5 text-base text-sky-300 hover:bg-sky-400/10 disabled:opacity-40"
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
            className="rounded-full border border-slate-700 bg-slate-900/60 px-2.5 py-1 text-base text-slate-300 hover:border-sky-400/60 hover:text-sky-200 disabled:opacity-40"
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
    <article className="rounded-lg border border-slate-800 bg-slate-900/60 p-3">
      <p className="text-base text-slate-100">{env.answer}</p>
      <div className="mt-2">
        <PathBadges path={env.path} />
      </div>
      <div className="mt-2 flex flex-wrap items-center gap-2 text-base">
        <span className="rounded bg-slate-800 px-1.5 py-0.5 font-mono text-slate-300">{env.template}</span>
        <span className="text-slate-400">{env.hops} hops</span>
        <span className="text-slate-500">·</span>
        <span className="text-slate-400">
          FalkorDB <span className="font-mono text-emerald-300">{env.run_time_ms} ms</span>
        </span>
        <span className="ml-auto">
          <GraphProofDrawer env={env} />
        </span>
      </div>
    </article>
  )
}
