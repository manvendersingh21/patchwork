import { useEffect, useRef, type ReactNode } from 'react'
import gsap from 'gsap'

export type WinRect = { x: number; y: number; w: number; h: number }

type WinProps = {
  title: string
  open: boolean
  minimized: boolean
  focused: boolean
  rect: WinRect
  onPointerDown: () => void
  onClose: () => void
  onMinimize: () => void
  children: ReactNode
}

const EDGE_PAD = 8
const BAR_KEEP = 44 // keep at least this much of the title bar on screen

/**
 * A desktop window: draggable by its title bar, GSAP open/close/minimize
 * animations, always mounted so closing animations can finish (the element is
 * display:none'd between runs, exactly like the reference site).
 */
export default function Win({
  title,
  open,
  minimized,
  focused,
  rect,
  onPointerDown,
  onClose,
  onMinimize,
  children,
}: WinProps) {
  const shellRef = useRef<HTMLDivElement>(null)
  const shownRef = useRef(false)
  const posRef = useRef({ x: rect.x, y: rect.y })

  // Keep the element mounted; toggle visibility with GSAP like the reference.
  useEffect(() => {
    const el = shellRef.current
    if (!el) return
    const shouldShow = open && !minimized
    if (shouldShow && !shownRef.current) {
      shownRef.current = true
      gsap.set(el, { display: 'flex', x: posRef.current.x, y: posRef.current.y })
      gsap.fromTo(
        el,
        { autoAlpha: 0, scale: 0.94 },
        { autoAlpha: 1, scale: 1, duration: 0.3, ease: 'back.out(1.5)' },
      )
    } else if (!shouldShow && shownRef.current) {
      shownRef.current = false
      gsap.to(el, {
        autoAlpha: 0,
        scale: minimized ? 0.9 : 0.97,
        duration: minimized ? 0.25 : 0.2,
        ease: 'power2.in',
        onComplete: () => {
          gsap.set(el, { display: 'none' })
          gsap.set(el, { scale: 1 })
        },
      })
    }
  }, [open, minimized])

  // Drag by title bar; direct style writes during the move, no React state.
  const onBarPointerDown = (e: React.PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return
    if (e.target instanceof Element && e.target.closest('button, input, a')) return
    const el = shellRef.current
    if (!el) return
    onPointerDown()
    e.preventDefault()
    const sx = e.clientX
    const sy = e.clientY
    const ox = posRef.current.x
    const oy = posRef.current.y
    const pid = e.pointerId
    try {
      ;(e.currentTarget as HTMLDivElement).setPointerCapture(pid)
    } catch {
      /* ignore */
    }
    document.body.style.cursor = 'grabbing'
    const move = (ev: PointerEvent) => {
      const maxX = window.innerWidth - EDGE_PAD - Math.min(rect.w, 160)
      const maxY = window.innerHeight - BAR_KEEP - 36
      const x = Math.min(Math.max(EDGE_PAD, ox + ev.clientX - sx), Math.max(EDGE_PAD, maxX))
      const y = Math.min(Math.max(EDGE_PAD, oy + ev.clientY - sy), Math.max(EDGE_PAD, maxY))
      posRef.current = { x, y }
      gsap.set(el, { x, y })
    }
    const up = () => {
      document.body.style.cursor = ''
      window.removeEventListener('pointermove', move, true)
      window.removeEventListener('pointerup', up, true)
    }
    window.addEventListener('pointermove', move, true)
    window.addEventListener('pointerup', up, true)
  }

  return (
    <div
      ref={shellRef}
      onPointerDown={onPointerDown}
      style={{ left: 0, top: 0, width: rect.w, height: rect.h, display: 'none', position: 'fixed' }}
      className={`win-shell pointer-events-auto z-[1] flex-col overflow-hidden ${
        focused ? 'win-focused' : ''
      }`}
      role="dialog"
      aria-label={title}
    >
      <div
        onPointerDown={onBarPointerDown}
        onDoubleClick={onMinimize}
        className="win-titlebar flex h-9 shrink-0 cursor-grab select-none items-center gap-2 px-3"
      >
        <div className="flex items-center gap-1.5">
          <button
            onClick={onClose}
            aria-label={`Close ${title}`}
            className="flex h-3.5 w-3.5 items-center justify-center rounded-full bg-emerald-900/70 text-[0.55rem] font-bold leading-none text-slate-950 hover:bg-rose-400"
          >
            ✕
          </button>
          <button
            onClick={onMinimize}
            aria-label={`Minimize ${title}`}
            className="flex h-3.5 w-3.5 items-center justify-center rounded-full bg-emerald-900/70 text-[0.55rem] font-bold leading-none text-slate-950 hover:bg-amber-300"
          >
            –
          </button>
        </div>
        <span className={`font-mono text-base tracking-wide ${focused ? 'text-accent' : 'text-emerald-200/50'}`}>
          {title}
        </span>
      </div>
      <div className="min-h-0 flex-1 overflow-hidden">{children}</div>
    </div>
  )
}
