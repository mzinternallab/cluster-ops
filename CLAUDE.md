# Claude Code Instructions

After completing every task or subtask:
1. Run git add -A
2. Commit with conventional commit format: feat/fix/refactor/chore(scope): description
3. Push to origin main

Never leave uncommitted changes.

## Cross-Platform Requirements

This app runs on Windows, macOS, and Linux.
Every line of code must work on all three platforms.

Rules:
- Never hardcode any username, home directory, or user-specific path
- Always use dirs::home_dir() for home directory
- Always use which::which("kubectl") for binary discovery
- Never hardcode C:\Users\anything
- Use cfg!(windows) / cfg!(target_os = "macos") / cfg!(unix)
  for platform-specific logic
- Path separators: use std::path::PathBuf and .join() never
  hardcode / or \
- Environment variable separators:
  if cfg!(windows) { ";" } else { ":" }
- kubectl binary name:
  if cfg!(windows) { "kubectl.exe" } else { "kubectl" }
- Test mentally for all three platforms before committing any change
