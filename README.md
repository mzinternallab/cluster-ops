# ClusterOps

A native desktop Kubernetes management app with AI-powered analysis.

## Prerequisites

- Rust (via [rustup.rs](https://rustup.rs))
- Node.js LTS
- kubectl
- Git
- Platform-specific dependencies (see below)

---

## Windows Prerequisites

1. **Rust** — download `rustup-init.exe` from https://rustup.rs

2. **Node.js** — download the LTS installer from https://nodejs.org

3. **Visual Studio Build Tools**
   - Download from https://aka.ms/vs/17/release/vs_BuildTools.exe
   - Select **"Desktop development with C++"**

4. **kubectl** — download from https://kubernetes.io/docs/tasks/tools/install-kubectl-windows/
   Place in a folder on your PATH, e.g. `C:\tools\kubectl.exe`

---

## macOS Prerequisites

1. **Homebrew** (if not already installed):
   ```bash
   /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
   ```

2. **Dependencies**:
   ```bash
   brew install node kubectl
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Xcode Command Line Tools** (Tauri system dependency, already on most Macs):
   ```bash
   xcode-select --install
   ```

---

## Linux Prerequisites (Ubuntu/Debian)

1. **System dependencies**:
   ```bash
   sudo apt-get update
   sudo apt-get install -y \
     libwebkit2gtk-4.1-dev \
     libgtk-3-dev \
     librsvg2-dev \
     libssl-dev \
     pkg-config \
     build-essential \
     curl
   ```

2. **Rust**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.bashrc
   ```

3. **Node.js** via NVM:
   ```bash
   curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
   source ~/.bashrc
   nvm install --lts
   nvm use --lts
   ```

4. **kubectl**:
   ```bash
   curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
   chmod +x kubectl
   sudo mv kubectl /usr/local/bin/
   ```

---

## AI Provider Configuration

ClusterOps supports four AI backends for analysis features (pod security scan,
network scan, RBAC scan, describe analysis, log analysis). Configure via
environment variables before launching the app.

### Anthropic (default)

```bash
AI_PROVIDER=anthropic
AI_API_KEY=sk-ant-...
AI_MODEL=claude-sonnet-4-6        # optional — this is the default
```

### OpenAI

```bash
AI_PROVIDER=openai
AI_API_KEY=sk-...
AI_MODEL=gpt-4o                   # optional — this is the default
```

### Azure OpenAI

`AI_BASE_URL` must be the full endpoint URL including the deployment path and
`api-version` query parameter.

```bash
AI_PROVIDER=azure
AI_API_KEY=your-azure-key
AI_MODEL=gpt-4                    # optional — this is the default
AI_BASE_URL=https://your-resource.openai.azure.com/openai/deployments/gpt-4/chat/completions?api-version=2024-02-01
```

### Ollama (local, no API key needed)

```bash
AI_PROVIDER=ollama
AI_MODEL=llama3                   # optional — this is the default
AI_BASE_URL=http://localhost:11434  # optional — this is the default
```

> **Backwards compatibility:** If `AI_API_KEY` is not set, the app falls back
> to `ANTHROPIC_API_KEY`, so existing configurations continue to work without
> any changes.

### Setting environment variables

**Windows (PowerShell):**

```powershell
[System.Environment]::SetEnvironmentVariable("AI_PROVIDER", "anthropic", "User")
[System.Environment]::SetEnvironmentVariable("AI_API_KEY", "sk-ant-your-key-here", "User")
```

Close and reopen PowerShell after running this. Alternatively use Start → "Environment Variables" → User variables.

**macOS / Linux** — add to `~/.bashrc` or `~/.zshrc`:

```bash
export AI_PROVIDER=anthropic
export AI_API_KEY="sk-ant-your-key-here"
```

Then reload: `source ~/.bashrc`

---

## Connecting to ORNL Forerunner AI

ORNL operates an internal AI platform (Forerunner) running at https://forerunner.ornl.gov powered by Open WebUI. ClusterOps can use this internal model instead of external AI providers.

### Getting Your API Key

1. Navigate to https://forerunner.ornl.gov and log in with your ORNL credentials
2. Click your profile icon → Settings → Account
3. Scroll to the API Keys section
4. Click "Create new secret key" and copy the key

### Windows (PowerShell)

Run the following commands in PowerShell, replacing `your-secret-key-here` with your actual key:

```powershell
[System.Environment]::SetEnvironmentVariable("AI_PROVIDER", "openai", "User")
[System.Environment]::SetEnvironmentVariable("AI_MODEL", "AMD.llama-4-maverick", "User")
[System.Environment]::SetEnvironmentVariable("AI_API_KEY", "your-secret-key-here", "User")
[System.Environment]::SetEnvironmentVariable("AI_BASE_URL", "https://forerunner.ornl.gov/api", "User")
```

Close and reopen PowerShell after setting these variables, then restart ClusterOps.

### macOS / Linux

Add the following to your `~/.bashrc` or `~/.zshrc`:

```bash
export AI_PROVIDER=openai
export AI_MODEL=AMD.llama-4-maverick
export AI_API_KEY=your-secret-key-here
export AI_BASE_URL=https://forerunner.ornl.gov/api
```

Then run:

```bash
source ~/.bashrc   # or source ~/.zshrc
```

### Verifying the Connection

To verify the connection works before starting ClusterOps, run this curl command:

```bash
curl -X POST "https://forerunner.ornl.gov/api/chat/completions" \
  -H "Authorization: Bearer your-secret-key-here" \
  -H "Content-Type: application/json" \
  -d '{"model": "AMD.llama-4-maverick", "messages": [{"role": "user", "content": "hello"}], "stream": false}'
```

A successful response will return a JSON object with `choices` containing the model response.

### Notes

- `AI_PROVIDER` is set to `openai` because ORNL Forerunner uses an OpenAI-compatible API format
- The default model is `AMD.llama-4-maverick`
- Your secret key is specific to your ORNL account
- Contact the ORNL AI team if you need access or have authentication issues

---

## Kubeconfig Setup

Place kubeconfig files in `~/.kube/` named `config.<clustername>`:

| Platform      | Path example                                    |
|---------------|-------------------------------------------------|
| Windows       | `C:\Users\USERNAME\.kube\config.mycluster`      |
| macOS / Linux | `~/.kube/config.mycluster`                      |

ClusterOps auto-discovers all `config.*` files on startup. No `KUBECONFIG` environment variable is needed.

---

## Running in Development

```bash
git clone https://github.com/mzinternallab/cluster-ops.git
cd cluster-ops
npm install
npm run tauri dev
```

---

## Building for Production

```bash
npm run tauri build
```

Output locations:

| Platform | Path                                              |
|----------|---------------------------------------------------|
| Windows  | `src-tauri/target/release/bundle/msi/`            |
| macOS    | `src-tauri/target/release/bundle/dmg/`            |
| Linux    | `src-tauri/target/release/bundle/deb/` or `appimage/` |

---

## Adding a New Cluster

1. Download kubeconfig from Rancher or your cluster provider
2. Save to `~/.kube/config.<clustername>` (e.g. `~/.kube/config.production`)
3. Restart ClusterOps
4. New cluster appears automatically in the cluster dropdown



## Security Framework References
ClusterOps security scanning is built on the most widely adopted US Government and industry Kubernetes security frameworks, providing automated analysis aligned with federal compliance requirements. These frameworks represent the current gold standard for securing containerized workloads in government and enterprise environments, and are referenced by both the Department of Energy (DOE) and Department of Defense (DOD) for container security guidance.
Frameworks used in ClusterOps security scans:

NSA/CISA Kubernetes Hardening Guide (2022) — Published jointly by the National Security Agency and Cybersecurity and Infrastructure Security Agency, this guide provides specific technical recommendations for hardening Kubernetes clusters against malicious actors. It covers pod security, network separation, authentication, audit logging, and upgrade practices. ClusterOps pod, network, and RBAC scans directly reference controls from this guide.
NIST SP 800-190 — Application Container Security Guide — Published by the National Institute of Standards and Technology, this publication addresses security concerns associated with container technologies across the full lifecycle including images, registries, orchestrators, containers, and host operating systems. It is referenced by both DOE and DOD container security policies.

CIS Kubernetes Benchmark v1.8 — Published by the Center for Internet Security, this benchmark provides consensus-based security configuration guidelines for Kubernetes. It is widely adopted across government agencies and maps directly to NIST controls. ClusterOps uses CIS benchmark checks as the basis for pod security, namespace, and node security scanning.

NIST SP 800-53 Rev 5 — The catalog of security and privacy controls for federal information systems. ClusterOps RBAC scanning references relevant access control (AC) and least privilege controls from this publication when analyzing role bindings and service account permissions.


