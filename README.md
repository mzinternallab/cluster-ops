# ClusterOps

A native desktop Kubernetes management app with AI-powered analysis.

## Prerequisites

- kubectl installed and in PATH
- kubeconfig files in `~/.kube/` named `config.<clustername>`
  - Example: `~/.kube/config.eagle-i-orc`, `~/.kube/config.rovi`
- Anthropic API key (for AI analysis feature)

## Setting up the Anthropic API Key

### Windows

**Option 1 — PowerShell:**
```powershell
[System.Environment]::SetEnvironmentVariable("ANTHROPIC_API_KEY", "sk-ant-your-key-here", "User")
```

**Option 2 — GUI:**
1. Open Start → search "Environment Variables"
2. Click "Edit the system environment variables"
3. Click "Environment Variables"
4. Under "User variables" click "New"
5. Variable name: `ANTHROPIC_API_KEY`
6. Variable value: `sk-ant-your-key-here`
7. Click OK

### macOS / Linux

Add to `~/.bashrc` or `~/.zshrc`:
```bash
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

Then reload:
```bash
source ~/.bashrc
```

## Running the App (Development)

### Windows (PowerShell)
```powershell
cd cluster-ops
npm install
npm run tauri dev
```

### macOS / Linux
```bash
cd cluster-ops
npm install
npm run tauri dev
```

## Adding a New Cluster

1. Download kubeconfig from Rancher or your cluster provider
2. Save it to `~/.kube/config.<clustername>`
   - Example: `~/.kube/config.production`
3. Restart ClusterOps
4. New cluster appears automatically in the cluster dropdown

## Building for Production

```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/
```
