import sithLogo from '../../assets/sith-logo.png'
import { ClusterSelector } from './ClusterSelector'
import { WindowControls } from './WindowControls'

// ── App logo ──────────────────────────────────────────────────────────────────

function AppLogo() {
  return (
    <div className="flex items-center gap-2 px-3 shrink-0 select-none">
      <img
        src={sithLogo}
        alt="ClusterOps"
        className="h-6 w-6 object-contain"
      />
    </div>
  )
}

// ── TitleBar ──────────────────────────────────────────────────────────────────

export function TitleBar() {
  return (
    <div
      data-tauri-drag-region
      className="flex items-center h-10 border-b border-border select-none shrink-0"
      style={{ background: '#1a0000' }}
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
