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

## Anthropic API Key

The AI analysis feature requires an Anthropic API key.

### Windows (PowerShell)

```powershell
[System.Environment]::SetEnvironmentVariable("ANTHROPIC_API_KEY", "sk-ant-your-key-here", "User")
```

Close and reopen PowerShell after running this.

Alternatively via the GUI: Start → search "Environment Variables" → Edit the system environment variables → Environment Variables → User variables → New → `ANTHROPIC_API_KEY`.

### macOS / Linux

Add to `~/.bashrc` or `~/.zshrc`:

```bash
export ANTHROPIC_API_KEY="sk-ant-your-key-here"
```

Then reload:

```bash
source ~/.bashrc
```

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
