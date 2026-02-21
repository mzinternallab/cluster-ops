import { useCallback, useEffect, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { cn } from '@/lib/utils'

// Inline SVG icons — keeps window chrome dependency-free
function MinimizeIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden>
      <path d="M1 5h8" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
    </svg>
  )
}

function MaximizeIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden>
      <rect x="1" y="1" width="8" height="8" rx="0.5" stroke="currentColor" strokeWidth="1.2" />
    </svg>
  )
}

function RestoreIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden>
      <rect x="3" y="1" width="6" height="6" rx="0.5" stroke="currentColor" strokeWidth="1.2" />
      <path
        d="M1 4v4.5A.5.5 0 0 0 1.5 9H6"
        stroke="currentColor"
        strokeWidth="1.2"
        strokeLinecap="round"
      />
    </svg>
  )
}

function CloseIcon() {
  return (
    <svg width="10" height="10" viewBox="0 0 10 10" fill="none" aria-hidden>
      <path d="M1.5 1.5l7 7M8.5 1.5l-7 7" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
    </svg>
  )
}

interface WinBtnProps {
  onClick: () => void
  className?: string
  'aria-label': string
  children: React.ReactNode
}

function WinBtn({ onClick, className, 'aria-label': ariaLabel, children }: WinBtnProps) {
  return (
    <button
      onClick={onClick}
      aria-label={ariaLabel}
      className={cn(
        'flex items-center justify-center w-11 h-10 text-text-muted transition-colors duration-100',
        'hover:text-text-primary focus-visible:outline-none',
        className,
      )}
    >
      {children}
    </button>
  )
}

export function WindowControls() {
  const [isMaximized, setIsMaximized] = useState(false)

  const refreshMaximized = useCallback(async () => {
    const win = getCurrentWindow()
    setIsMaximized(await win.isMaximized())
  }, [])

  useEffect(() => {
    const win = getCurrentWindow()
    let unlisten: (() => void) | undefined

    refreshMaximized()

    win.onResized(refreshMaximized).then((fn) => {
      unlisten = fn
    })

    return () => {
      unlisten?.()
    }
  }, [refreshMaximized])

  const win = getCurrentWindow()

  return (
    <div className="flex items-center shrink-0 ml-2">
      <WinBtn onClick={() => void win.minimize()} aria-label="Minimize" className="hover:bg-white/8">
        <MinimizeIcon />
      </WinBtn>
      <WinBtn
        onClick={() => void (isMaximized ? win.unmaximize() : win.maximize())}
        aria-label={isMaximized ? 'Restore' : 'Maximize'}
        className="hover:bg-white/8"
      >
        {isMaximized ? <RestoreIcon /> : <MaximizeIcon />}
      </WinBtn>
      <WinBtn onClick={() => void win.close()} aria-label="Close" className="hover:bg-error hover:text-white">
        <CloseIcon />
      </WinBtn>
    </div>
  )
}
