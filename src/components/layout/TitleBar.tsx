// Custom title bar with window controls and cluster switcher
// Implemented in Phase 1 Step 2

export function TitleBar() {
  return (
    <div
      data-tauri-drag-region
      className="flex items-center justify-between h-10 px-3 bg-surface border-b border-border select-none shrink-0"
    >
      <div className="flex items-center gap-2">
        <span className="text-accent font-mono text-xs font-semibold tracking-widest uppercase">
          cluster-ops
        </span>
      </div>
      {/* Cluster switcher — Phase 1 Step 2 */}
      <div className="flex items-center gap-1">
        <span className="text-text-muted text-xxs">No cluster connected</span>
      </div>
      {/* Window controls — Phase 1 Step 2 */}
      <div className="flex items-center gap-1" data-tauri-drag-region="false">
        <button className="w-3 h-3 rounded-full bg-yellow-400 hover:bg-yellow-300" aria-label="Minimize" />
        <button className="w-3 h-3 rounded-full bg-green-500 hover:bg-green-400" aria-label="Maximize" />
        <button className="w-3 h-3 rounded-full bg-red-500 hover:bg-red-400" aria-label="Close" />
      </div>
    </div>
  )
}
