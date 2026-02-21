# KubeOps — Project Specification

> **Version:** 0.1.0  
> **Last Updated:** 2026-02-21  
> **Purpose:** This document is the authoritative reference for building KubeOps. Claude Code should follow this spec when scaffolding, implementing features, and making architectural decisions.

---

## 1. Project Overview

KubeOps is a native desktop application for Kubernetes operations. It provides a graphical, k9s-inspired interface for managing multiple clusters, viewing workloads, running kubectl commands, and getting AI-powered analysis of pod output and logs — all from a single unified desktop window.

**Elevator pitch:** "k9s, but with a beautiful GUI and an AI co-pilot built in."

---

## 2. Core Goals

| Goal | Description |
|------|-------------|
| True desktop app | Not a web UI. Native window via Tauri 2.0 |
| Multi-cluster | Switch between clusters using local kubeconfig |
| Workload visibility | View and interact with pods, deployments, configmaps, secrets |
| kubectl integration | Run describe/logs commands with output rendered in-app |
| AI analysis | Auto-pipe kubectl output to Claude API for error detection and suggestions |
| Expandable | Clean architecture so new resource types and views are easy to add |
| Cross-platform | Windows (primary), Linux, macOS |
| Low overhead | Tauri over Electron. Target <50MB RAM idle |

---

## 3. Tech Stack

### Desktop Framework
- **Tauri 2.0** — native desktop shell, system tray, OS integration, auto-updater
- WebView2 on Windows, WKWebView on macOS, WebKitGTK on Linux

### Frontend
- **React 18** with **TypeScript**
- **Vite** as the build tool
- **Tailwind CSS v3** for utility styling
- **shadcn/ui** for base component library
- **xterm.js** for embedded terminal output rendering
- **Zustand** for global state management
- **React Query (TanStack Query)** for async data fetching and caching

### Backend (Rust / Tauri Commands)
- **kube-rs** — Kubernetes API client (direct API calls, no kubectl dependency for data)
- **tokio** — async runtime
- **serde / serde_json** — serialization
- Tauri Commands expose Rust functions to the frontend via `invoke()`

### AI Integration
- **Anthropic Claude API** (claude-sonnet-4-6 model)
- Called from Rust backend via `reqwest` HTTP client
- API key stored in OS keychain via Tauri's `keyring` plugin
- Streaming responses via SSE

### Dev Tools
- **ESLint + Prettier** for frontend linting
- **Clippy + rustfmt** for Rust linting
- **Vitest** for frontend unit tests

---

## 4. Repository Structure

```
kubeops/
├── src/                          # React frontend
│   ├── main.tsx                  # Entry point
│   ├── App.tsx                   # Root component, layout
│   ├── components/
│   │   ├── layout/
│   │   │   ├── TitleBar.tsx      # Custom title bar with cluster switcher
│   │   │   ├── Sidebar.tsx       # Left nav (Workloads, Configs, Secrets, etc.)
│   │   │   └── NamespaceBar.tsx  # Namespace filter bar
│   │   ├── workloads/
│   │   │   ├── PodTable.tsx      # Main pods table
│   │   │   ├── PodRow.tsx        # Individual pod row
│   │   │   └── StatusBadge.tsx   # Status pill (Running/CrashLoop/etc.)
│   │   ├── terminal/
│   │   │   ├── OutputPanel.tsx   # kubectl output terminal panel
│   │   │   └── CommandBar.tsx    # kubectl command input bar
│   │   ├── ai/
│   │   │   ├── AIPanel.tsx       # AI analysis sidebar panel
│   │   │   ├── AIInsight.tsx     # Individual insight card
│   │   │   └── AILoader.tsx      # Streaming loader animation
│   │   └── ui/                   # shadcn/ui components
│   ├── views/
│   │   ├── WorkloadsView.tsx     # Pods + Deployments + ReplicaSets
│   │   ├── ConfigMapsView.tsx    # ConfigMaps table
│   │   ├── SecretsView.tsx       # Secrets table (values masked)
│   │   ├── NetworkView.tsx       # Services + Ingresses (future)
│   │   └── StorageView.tsx       # PVCs + PVs (future)
│   ├── hooks/
│   │   ├── useCluster.ts         # Cluster switching, kubeconfig parsing
│   │   ├── usePods.ts            # Pod list fetching and polling
│   │   ├── useKubectl.ts         # Run kubectl commands, stream output
│   │   └── useAI.ts              # Send output to AI, stream response
│   ├── store/
│   │   ├── clusterStore.ts       # Active cluster, available clusters
│   │   ├── namespaceStore.ts     # Active namespace filter
│   │   └── uiStore.ts            # Panel visibility, selected pod, etc.
│   ├── types/
│   │   ├── kubernetes.ts         # TypeScript types for K8s resources
│   │   └── ai.ts                 # AI response types
│   └── styles/
│       └── globals.css           # Tailwind base + custom CSS variables
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri app entry point
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── kubeconfig.rs     # List clusters from kubeconfig
│   │   │   ├── pods.rs           # List/get pods via kube-rs
│   │   │   ├── kubectl.rs        # Spawn kubectl, stream output
│   │   │   ├── logs.rs           # Stream pod logs
│   │   │   └── ai.rs             # Call Anthropic API, stream response
│   │   └── models/
│   │       ├── mod.rs
│   │       └── k8s.rs            # Rust structs for K8s data
│   ├── Cargo.toml
│   └── tauri.conf.json
├── SPEC.md                       # This file
├── README.md
├── package.json
├── vite.config.ts
├── tailwind.config.ts
└── tsconfig.json
```

---

## 5. Feature Specification — Phase 1 (MVP)

### 5.1 Cluster Switcher

**Location:** Title bar, top-right  
**Behavior:**
- On startup, read `~/.kube/config` (and any paths in `KUBECONFIG` env var)
- Parse all available contexts
- Display as tabs/buttons in the title bar
- Each cluster shows a colored health dot (green = reachable, yellow = slow, red = unreachable)
- Switching cluster re-fetches all resource data for that cluster
- Active context is persisted between app restarts

**Tauri Command:** `get_kubeconfig_contexts() -> Vec<KubeContext>`  
**Tauri Command:** `set_active_context(context_name: String) -> Result<()>`

```typescript
interface KubeContext {
  name: string;
  cluster: string;
  user: string;
  namespace?: string;
  isActive: boolean;
}
```

---

### 5.2 Workloads View — Pods Table

**Location:** Main content area when "Workloads" is selected in sidebar  
**Columns:** NAME · NAMESPACE · STATUS · READY · RESTARTS · AGE · CPU · MEM · NODE

**Behavior:**
- Fetch pod list via kube-rs (not kubectl subprocess) every 10 seconds
- Filter by selected namespace (or show all)
- Click a row to select it (highlights the row)
- Status column shows colored badge: Running=green, CrashLoopBackOff=red, Pending=yellow, Terminating=orange
- Restarts > 5 shown in red bold
- "describe" and "logs" action buttons on each row
- Sortable columns (click header to sort)
- Search/filter input to filter pods by name

**Tauri Command:** `list_pods(namespace: Option<String>) -> Vec<PodSummary>`

```typescript
interface PodSummary {
  name: string;
  namespace: string;
  status: string;
  ready: string;
  restarts: number;
  age: string;
  cpu: string;
  memory: string;
  node: string;
  labels: Record<string, string>;
}
```

---

### 5.3 Namespace Selector

**Location:** Below the title bar, above the pod table  
**Behavior:**
- Show all namespaces as filter pills
- "all" is the default
- Selecting a namespace filters the pod table
- Namespace list fetched from the cluster on startup and on cluster switch

**Tauri Command:** `list_namespaces() -> Vec<String>`

---

### 5.4 Output Panel — kubectl describe / logs

**Location:** Bottom split panel, appears when user clicks "describe" or "logs"  
**Behavior:**
- Panel slides up from bottom, splitting the screen
- Top half: pod table (scrollable, compressed)
- Bottom half: output panel + AI panel side by side
- Output rendered using xterm.js for authentic terminal look
- ANSI color codes supported
- Auto-scrolls to bottom as output streams in
- Error lines (containing "Error", "FATAL", "OOMKill", "Warning", "BackOff") highlighted in red/yellow automatically
- Close button (×) collapses the panel

**For `describe`:**
- Call `kubectl describe pod <name> -n <namespace>`
- Output streams line by line into terminal panel

**For `logs`:**
- Call `kubectl logs <name> -n <namespace> --tail=200 -f`
- Streaming logs, line by line
- "Follow" toggle to keep streaming vs just fetch last N lines
- "Lines" selector: last 50 / 100 / 200 / 500 / all

**Tauri Command:** `describe_pod(name: String, namespace: String) -> Result<String>`  
**Tauri Command:** `get_pod_logs(name: String, namespace: String, tail: u32, follow: bool)`  
(logs uses Tauri event streaming, not a return value)

---

### 5.5 AI Analysis Panel

**Location:** Right side of the output panel (340px wide sidebar)  
**Behavior:**
- Automatically triggers when output panel opens
- Shows loading animation while waiting for Claude API response
- Streams response tokens in as they arrive
- Displays structured insights:
  - 🔴 **Critical** — errors, crashes, OOM kills, CrashLoopBackOff causes
  - 🟡 **Warning** — high restart counts, resource pressure, slow responses
  - 💡 **Suggestion** — recommended kubectl commands or config changes to fix the issue
- Each suggestion includes a copyable kubectl command where applicable
- "Re-analyze" button to re-run analysis
- Can be hidden/shown with a toggle

**AI Prompt for describe output:**
```
You are a Kubernetes operations expert. Analyze the following kubectl describe output and identify:
1. Any errors, crashes, or critical issues
2. Warnings or concerning patterns  
3. Specific actionable kubectl commands to fix any issues found

Format your response as JSON with this structure:
{
  "insights": [
    {
      "type": "critical" | "warning" | "suggestion",
      "title": "Short title",
      "body": "Explanation",
      "command": "kubectl command if applicable (optional)"
    }
  ]
}

kubectl describe output:
{OUTPUT}
```

**AI Prompt for logs:**
```
You are a Kubernetes operations expert. Analyze the following pod logs and identify:
1. Any errors, crashes, panics, or fatal issues
2. Warnings or concerning patterns
3. Root cause analysis if possible
4. Specific actionable suggestions to fix any issues

Format your response as JSON with this structure:
{
  "insights": [
    {
      "type": "critical" | "warning" | "suggestion", 
      "title": "Short title",
      "body": "Explanation",
      "command": "kubectl command if applicable (optional)"
    }
  ]
}

Pod logs:
{OUTPUT}
```

**Tauri Command:** `analyze_with_ai(output: String, mode: String) -> Result<String>`  
(streams response via Tauri events)

**API Config:**
- Model: `claude-sonnet-4-6`
- Max tokens: 1024
- Temperature: 0 (deterministic for analysis)
- API key: read from OS keychain, set via Settings screen

---

### 5.6 Command Bar

**Location:** Right side of namespace bar  
**Behavior:**
- Text input that accepts raw kubectl commands
- Prefix `kubectl` is implied (user types `describe pod worker-abc`)
- Enter key executes the command
- Output opens in the output panel
- Command history (up/down arrow keys)
- Tab completion for pod names (from cached pod list)

---

## 6. Feature Specification — Phase 2 (Post-MVP)

These are planned but NOT in scope for the initial build. Architecture should accommodate them.

| Feature | Description |
|---------|-------------|
| ConfigMaps View | Table of configmaps with key/value viewer, edit + apply |
| Secrets View | Table of secrets, values masked by default, reveal toggle |
| Deployments View | Deployment list with rollout status, scale up/down buttons |
| Events View | Cluster events feed, filterable by namespace/type |
| Node View | Node list with resource usage, cordon/uncordon |
| Port Forward | GUI for `kubectl port-forward`, active tunnels list |
| Exec into Pod | Shell into pod via xterm.js full terminal |
| AI Chat | Natural language chat: "Why is my worker pod crashing?" |
| Helm View | List installed Helm releases, upgrade/rollback |
| Metrics | CPU/Memory graphs using metrics-server data |
| Settings | API key management, kubeconfig path, theme, polling interval |

---

## 7. UI Design Guidelines

### Theme
- **Dark only** (for MVP)
- Background: `#0a0e1a` (deep navy black)
- Surface: `#0d1117` (title bar, sidebar, headers)
- Border: `#1e2a3a`
- Primary text: `#c9d1e9`
- Muted text: `#4a6a8a`
- Accent blue: `#4a90d9`
- Success green: `#22c55e`
- Warning yellow: `#f59e0b`
- Error red: `#ef4444`
- AI purple: `#7a7adc`

### Typography
- **UI font:** `JetBrains Mono` or `Cascadia Code` (monospace throughout — it's a terminal tool)
- Font sizes: 10px (labels), 11px (table data), 12px (names), 13px (body), 16px (icons)

### Layout
- Custom title bar (Tauri `decorations: false`)
- Left sidebar: 52px icon-only nav
- Main content: full width pod table
- Bottom split: output panel + AI panel (toggled by actions)
- No scrollbars visible — use custom styled scrollbars

### Spacing
- 4px base unit
- Table row height: 32px
- Panel padding: 12px / 16px
- Border radius: 4px (buttons), 6px (panels)

---

## 8. Tauri Configuration

```json
{
  "productName": "KubeOps",
  "version": "0.1.0",
  "identifier": "com.kubeops.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "KubeOps",
        "width": 1400,
        "height": 900,
        "minWidth": 1000,
        "minHeight": 600,
        "decorations": false,
        "transparent": false
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.ico"]
  },
  "plugins": {
    "shell": { "open": true },
    "fs": { "all": true, "scope": ["$HOME/.kube/**"] }
  }
}
```

---

## 9. Environment Variables & Configuration

```
KUBECONFIG          Path(s) to kubeconfig file(s), colon-separated
ANTHROPIC_API_KEY   Claude API key (fallback if not in keychain)
KUBEOPS_LOG_LEVEL   debug | info | warn | error (default: info)
KUBEOPS_POLL_MS     Pod list polling interval in ms (default: 10000)
```

---

## 10. Build & Development

### Prerequisites
```powershell
winget install OpenJS.NodeJS        # Node.js 20+
winget install Rustlang.Rustup      # Rust toolchain
winget install Microsoft.EdgeWebView2Runtime
rustup target add x86_64-pc-windows-msvc
```

### Install & Run
```bash
npm install
npm run tauri dev
```

### Build for Production
```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/
```

### Run Tests
```bash
npm run test          # Frontend unit tests (Vitest)
cargo test            # Rust backend tests
```

---

## 11. Git Conventions

- Branch: `main` (production), `dev` (active development)
- Commit format: `type(scope): description`
  - Types: `feat`, `fix`, `refactor`, `style`, `docs`, `test`, `chore`
  - Examples: `feat(pods): add pod table with status badges`
  - Examples: `fix(ai): handle empty log output gracefully`
- PRs required for merging to main (even solo — for history clarity)

---

## 12. Phase 1 Build Order for Claude Code

Follow this order when building the MVP:

1. `chore: scaffold Tauri 2.0 + React + TypeScript + Tailwind project`
2. `feat(layout): custom title bar with window controls and cluster switcher UI`
3. `feat(layout): sidebar navigation and namespace filter bar`
4. `feat(rust): kubeconfig parser — read and list contexts`
5. `feat(cluster): wire cluster switcher to kubeconfig contexts`
6. `feat(rust): kube-rs pod listing command`
7. `feat(workloads): pod table component with status badges`
8. `feat(workloads): namespace filtering and pod search`
9. `feat(rust): kubectl describe and logs streaming commands`
10. `feat(terminal): xterm.js output panel with error highlighting`
11. `feat(ai): Anthropic API integration in Rust backend`
12. `feat(ai): AI analysis panel with streaming insights`
13. `feat(terminal): command bar with history and tab completion`
14. `fix + polish: error states, loading states, empty states`
15. `docs: update README with setup instructions`

---

*This spec is a living document. Update it as features are added or decisions change.*
