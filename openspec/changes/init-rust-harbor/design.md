## Context

The application is a brand new desktop app built on Tauri, utilizing Svelte and shadcn-svelte on the frontend, and Rust on the backend. It needs to monitor local Git repositories efficiently. The initial design originally featured SQLite, but was migrated to use direct file-system interrogation and memory state to prevent out-of-sync tracking bugs. A small configuration file (JSON/TOML) will persist user settings.

## Goals / Non-Goals

**Goals:**
- Provide a robust IPC architecture between Svelte and Rust.
- Set up automated `notify` file watchers scoped strictly to user-defined "Watched Folders".
- Present an intuitive, beautiful dashboard of Git repos via shadcn-svelte.
- Implement UI-based action triggers for Git commands (`git pull`, etc.).

**Non-Goals:**
- No full-fledged CLI terminal emulator inside the app.
- No heavy SQL database integrations.
- Managing codebases that are not Git repositories.

## Decisions

- **Tauri + Svelte + shadcn-svelte**: Hand-selected for the best blend of native system performance (Rust) and extremely rapid, beautiful UI prototyping (Svelte).
- **File System as Source of Truth**: Removing SQLite in favor of using the file system directly reduces architectural complexity and guarantees the app is never out of sync with terminal-based Git operations.
- **`git2` and Subprocessing**: For complicated Git interactions and metadata extraction, `git2` will be used. Where `git2` is insufficient or verbose, standard `std::process::Command` will invoke the system's `git` command directly.

## Risks / Trade-offs

- **Risk: OS File Handler Exhaustion**: Watching too many directories might exceed the OS file watcher limits (`ulimit`). 
  - **Mitigation**: The system restricts file watching strictly to user-designated "Watched Folders", and implements a fallback polling mechanism if file watchers fail.
- **Risk: Missing Git CLI**: `git` operations natively invoked fail if `git` is absent.
  - **Mitigation**: A strict pre-flight check runs upon app initialization validating `git` is available in `PATH`, rendering an explicit blocking error state if it isn't.
