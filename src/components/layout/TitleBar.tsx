import { ClusterSelector } from './ClusterSelector'
import { WindowControls } from './WindowControls'

// ── App logo ──────────────────────────────────────────────────────────────────

function AppLogo() {
  return (
    <div className="flex items-center gap-2 px-3 shrink-0 select-none">
      <svg width="16" height="16" viewBox="0 0 16 16" fill="none" aria-hidden>
        <path
          d="M8 1.5L14 5v6L8 14.5 2 11V5L8 1.5z"
          stroke="#4a90d9"
          strokeWidth="1.2"
          strokeLinejoin="round"
        />
        <circle cx="8" cy="8" r="1.5" fill="#4a90d9" />
        <path
          d="M8 3.5v2M8 10.5v2M4.5 5.75l1.75 1M9.75 9.25l1.75 1M4.5 10.25l1.75-1M9.75 6.75l1.75-1"
          stroke="#4a90d9" strokeWidth="1" strokeLinecap="round"
        />
      </svg>
      <span className="text-xs font-semibold tracking-widest text-text-muted uppercase select-none">
        cluster-ops
      </span>
    </div>
  )
}

// ── TitleBar ──────────────────────────────────────────────────────────────────

export function TitleBar() {
  return (
    <div
      data-tauri-drag-region
      className="flex items-center h-10 bg-surface border-b border-border select-none shrink-0"
    >
      {/* Left: branding */}
      <AppLogo />

      {/* Separator */}
      <div className="w-px h-5 bg-border shrink-0" />

      {/* Center: cluster dropdown */}
      <div className="flex flex-1 items-center px-3">
        <ClusterSelector />
      </div>

      {/* Right: window controls */}
      <WindowControls />
    </div>
  )
}
