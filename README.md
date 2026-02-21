# cluster-ops

> k9s, but with a beautiful GUI and an AI co-pilot built in.

A native desktop application for Kubernetes operations. Provides a graphical interface for managing multiple clusters, viewing workloads, running kubectl commands, and getting AI-powered analysis of pod output and logs — all from a single unified window.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop shell | Tauri 2.0 |
| Frontend | React 18 + TypeScript + Vite |
| Styling | Tailwind CSS v3 + shadcn/ui |
| State | Zustand + TanStack Query |
| Backend | Rust (kube-rs, tokio, serde) |
| AI | Anthropic Claude API (claude-sonnet-4-6) |

## Prerequisites

```bash
# Node.js 20+
node --version

# Rust toolchain
rustup --version

# Linux: GTK + WebKit deps
sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/
```

## Tests

```bash
npm test          # Frontend unit tests (Vitest)
cargo test        # Rust backend tests (run from src-tauri/)
```

## Project Structure

```
src/                      # React frontend
  components/
    layout/               # TitleBar, Sidebar, NamespaceBar
    workloads/            # PodTable, PodRow, StatusBadge
    terminal/             # OutputPanel, CommandBar
    ai/                   # AIPanel, AIInsight, AILoader
    ui/                   # shadcn/ui components
  views/                  # WorkloadsView, ConfigMapsView, ...
  hooks/                  # useCluster, usePods, useKubectl, useAI
  store/                  # Zustand stores (cluster, namespace, ui)
  types/                  # TypeScript types (kubernetes.ts, ai.ts)
  styles/                 # globals.css (Tailwind + CSS vars)
src-tauri/                # Rust backend
  src/
    commands/             # Tauri commands (kubeconfig, pods, kubectl, logs, ai)
    models/               # Rust structs (k8s.rs)
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `KUBECONFIG` | Path(s) to kubeconfig file(s), colon-separated |
| `ANTHROPIC_API_KEY` | Claude API key (fallback if not in OS keychain) |
| `KUBEOPS_LOG_LEVEL` | `debug` \| `info` \| `warn` \| `error` (default: `info`) |
| `KUBEOPS_POLL_MS` | Pod list polling interval in ms (default: `10000`) |

## Build Phases

See [SPEC.md](./SPEC.md) for the full specification and phase build order.

- **Phase 1 (MVP):** Cluster switcher, pod table, kubectl output panel, AI analysis
- **Phase 2:** ConfigMaps, Secrets, Deployments, Events, Node view, Port Forward, Exec, AI Chat, Helm, Metrics
