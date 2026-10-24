import type { ReactNode } from 'react'

/**
 * A desktop icon: tile + label, reference-style hover/selection.
 * Double-click or Enter opens the window.
 */
export default function DesktopIcon({
  label,
  onOpen,
  children,
}: {
  label: string
  onOpen: () => void
  children: ReactNode
}) {
  return (
    <button
      type="button"
      onDoubleClick={onOpen}
      onClick={(e) => {
        // Single click on touch devices should open too.
        if (e.detail === 1 && window.matchMedia('(pointer: coarse)').matches) onOpen()
      }}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault()
          onOpen()
        }
      }}
      className="dicon group flex w-[7.5rem] flex-col items-center gap-1 p-1 outline-none"
      aria-label={`Open ${label}`}
    >
      <span className="dicon-tile flex h-16 w-16 items-center justify-center text-slate-600 group-hover:text-slate-900">
        {children}
      </span>
      <span className="dicon-label max-w-full truncate text-center font-mono text-base text-slate-600 group-hover:text-white">
        {label}
      </span>
    </button>
  )
}
