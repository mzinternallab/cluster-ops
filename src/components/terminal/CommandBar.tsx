// kubectl command input bar — Phase 1 Step 13
export function CommandBar() {
  return (
    <div className="flex items-center h-9 px-4 bg-surface border-t border-border">
      <span className="text-accent text-xs mr-2">kubectl</span>
      <input
        className="flex-1 bg-transparent text-xs text-text-primary outline-none placeholder:text-text-muted"
        placeholder="describe pod <name> -n <namespace>"
        disabled
      />
    </div>
  )
}
