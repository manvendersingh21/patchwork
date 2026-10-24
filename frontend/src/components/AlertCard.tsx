import { useState } from 'react'
import { api } from '../api'
import type { Alert } from '../types'

export default function AlertCard({ alert, onDismiss }: { alert: Alert; onDismiss: () => void }) {
  const [copied, setCopied] = useState(false)
  const [draft, setDraft] = useState(alert.draft)
  const [send, setSend] = useState<'idle' | 'confirm' | 'sending' | 'sent' | 'error'>('idle')
  const [sendError, setSendError] = useState('')

  const sendNow = async () => {
    setSend('sending')
    try {
      const r = await api.sendFollowup(alert.bridge_uid, draft)
      if (r.ok) setSend('sent')
      else {
        setSendError(r.error ?? 'send failed')
        setSend('error')
      }
    } catch (e) {
      setSendError(String(e))
      setSend('error')
    }
  }

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(draft)
      setCopied(true)
      setTimeout(() => setCopied(false), 1500)
    } catch {
      setCopied(false)
    }
  }

  return (
    <div className="toast-in w-[26rem] max-w-[calc(100vw-2rem)] rounded-lg border border-emerald-400/40 bg-[#04140d]/95 p-3 shadow-xl shadow-emerald-500/10">
      <h3 className="text-lg font-semibold text-emerald-300">Someone you met can help with this</h3>
      <p className="mt-1 text-base text-slate-100">
        <span className="font-medium">{alert.contact}</span> · {alert.place} ·{' '}
        {alert.met_at ? new Date(alert.met_at).toLocaleString() : ''}
      </p>
      <p className="mt-1 text-base italic text-emerald-100/60">&ldquo;{alert.note}&rdquo;</p>
      <p className="mt-2 text-base text-emerald-100/60">{alert.issue}</p>
      <textarea
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        disabled={send === 'sending' || send === 'sent'}
        rows={5}
        className="mt-2 w-full resize-y rounded border border-emerald-900/60 bg-emerald-950/50 p-2 text-base text-slate-100"
      />
      <div className="mt-2 flex items-center gap-2">
        <button
          onClick={() => void copy()}
          className="rounded border border-emerald-400/50 px-2.5 py-1 text-base text-emerald-300 hover:bg-emerald-400/10"
        >
          {copied ? 'Copied ✓' : 'Copy'}
        </button>
        {send === 'idle' && (
          <button
            onClick={() => setSend('confirm')}
            className="rounded border border-sky-400/50 px-2.5 py-1 text-base text-sky-300 hover:bg-sky-400/10"
          >
            Send on LinkedIn
          </button>
        )}
        {send === 'confirm' && (
          <button
            onClick={() => void sendNow()}
            className="rounded border border-amber-400/60 bg-amber-400/10 px-2.5 py-1 text-base text-amber-200"
          >
            Confirm send to {alert.contact}
          </button>
        )}
        {send === 'sending' && <span className="text-base text-sky-300">Sending…</span>}
        {send === 'sent' && <span className="text-base text-emerald-300">Sent on LinkedIn ✓</span>}
        <button
          onClick={onDismiss}
          className="rounded border border-emerald-900/60 px-2.5 py-1 text-base text-emerald-100/70 hover:bg-emerald-950/60"
        >
          Dismiss
        </button>
        <span className="ml-auto font-mono text-base text-emerald-100/70">
          {alert.hops} hops
        </span>
      </div>
      {send === 'error' && (
        <p className="mt-2 text-base text-rose-300">
          Not sent: {sendError}{' '}
          <button className="underline" onClick={() => setSend('idle')}>
            Try again
          </button>
        </p>
      )}
    </div>
  )
}
