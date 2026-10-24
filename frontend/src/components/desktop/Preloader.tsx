import { useEffect, useRef, useState } from 'react'
import gsap from 'gsap'

/**
 * Boot preloader in the reference's style: wordmark, thin loading bar,
 * marquee strip underneath, then GSAP fade-out. onDone fires when it's gone.
 */
export default function Preloader({ onDone }: { onDone: () => void }) {
  const rootRef = useRef<HTMLDivElement>(null)
  const firedRef = useRef(false)
  const [hidden, setHidden] = useState(false)

  useEffect(() => {
    const t = setTimeout(() => {
      if (firedRef.current) return
      firedRef.current = true
      const el = rootRef.current
      if (el) {
        gsap.to(el, {
          autoAlpha: 0,
          duration: 0.5,
          ease: 'power2.inOut',
          onComplete: () => {
            setHidden(true)
            onDone()
          },
        })
      } else {
        setHidden(true)
        onDone()
      }
    }, 1500)
    return () => clearTimeout(t)
  }, [onDone])

  if (hidden) return null

  return (
    <div
      ref={rootRef}
      className="fixed inset-0 z-[999] flex flex-col items-center justify-center bg-desktop"
    >
      <div className="font-mono text-4xl font-bold tracking-[0.35em] text-slate-100">
        PATCH<span className="text-accent">WORK</span>
      </div>
      <p className="mt-2 font-mono text-base tracking-[0.2em] text-slate-500">
        CONFERENCE MEMORY · FALKORDB
      </p>
      <div className="mt-6 h-px w-64 overflow-hidden bg-slate-800">
        <div className="load-bar-fill h-full bg-accent" />
      </div>
      <div className="absolute bottom-6 left-0 right-0 overflow-hidden border-y border-slate-800/80 py-1">
        <div className="marquee-track font-mono text-base text-slate-600">
          {[0, 1].map((k) => (
            <span key={k}>
              {Array.from({ length: 6 }).map((_, i) => (
                <span key={i} className="mx-6">
                  CARTOGRAPHER · PATHFINDER · CONCIERGE
                </span>
              ))}
            </span>
          ))}
        </div>
      </div>
    </div>
  )
}
