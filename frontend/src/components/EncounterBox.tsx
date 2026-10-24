import { useState } from 'react'
import { api } from '../api'

export default function EncounterBox() {
  const [note, setNote] = useState('')
  const [place, setPlace] = useState('')
  const [status, setStatus] = useState<string | null>(null)
  const [busy, setBusy] = useState(false)

  const submit = async () => {
    if (!note.trim() || busy) return
    setBusy(true)
    try {
      const res = await api.encounter(note.trim(), place.trim() || 'Universe hallway')
      setStatus(`✓ logged ${res.encounter_uid} — Cartographer parsing…`)
      setNote('')
      setPlace('')
    } catch {
      setStatus('✗ could not log the encounter')
    } finally {
      setBusy(false)
    }
  }

  return (
    <section className="space-y-1.5">
      <h2 className="text-lg font-semibold text-slate-100">Log a hallway conversation</h2>
      <textarea
        value={note}
        onChange={(e) => setNote(e.target.value)}
        rows={2}
        placeholder="Met Jordan at the snack table. They scope agent tokens so agents can open PRs but never merge them."
        className="w-full resize-none rounded border border-emerald-900/70 bg-emerald-950/50 px-2 py-1.5 text-base text-slate-50 placeholder:text-emerald-200/60 focus:border-emerald-400 focus:outline-none"
      />
      <div className="flex gap-2">
        <input
          value={place}
          onChange={(e) => setPlace(e.target.value)}
          placeholder="Where (snack table, Expo Hall…)"
          className="min-w-0 flex-1 rounded border border-emerald-900/70 bg-emerald-950/50 px-2 py-1.5 text-base text-slate-50 placeholder:text-emerald-200/60 focus:border-emerald-400 focus:outline-none"
        />
        <button
          onClick={() => void submit()}
          disabled={busy || !note.trim()}
          className="shrink-0 rounded border border-emerald-400/60 px-2.5 py-1.5 text-base text-emerald-200 hover:bg-emerald-400/10 disabled:opacity-40"
        >
          {busy ? 'Logging…' : 'Log'}
        </button>
      </div>
      {status && <p className="text-base text-emerald-100/60">{status}</p>}
    </section>
  )
}
