## Why

Managing multiple Git repositories manually across different folders and remotes requires constant CLI usage and directory traversal. Rust Harbor aims to provide a fast, centralized hub to visualize, watch, and interact with multiple Git projects from a single desktop UI. Leveraging a modern Rust + Tauri + Svelte stack, the application will be extremely responsive, cross-platform, and beautiful, strictly relying on the filesystem rather than an intermediary database to avoid state synchronization bugs.

## What Changes

This change involves initializing "Rust Harbor", a brand new local desktop app. Key additions include:
- A local scanning mechanism that selectively watches specified folders.
- A beautiful, responsive Svelte/shadcn-svelte frontend UI for the dashboard and Git operations.
- Interfacing directly with `git2` and system `git` CLI for underlying operations.
- File system watchers and optimized background polling to maintain an up-to-date UI without locking files or hogging system resources.

## Capabilities

### New Capabilities
- `watched-folders`: Managing the list of root folders that the application is allowed to scan and watch, persisting state via a local JSON/TOML configuration file.
- `local-repo-dashboard`: Displaying the discovered repositories (name, remote, branch, status) in real-time, backed by `notify` file watchers and restricted background polling.
- `ui-git-actions`: Dedicated UI buttons/menus to perform `git fetch`, `git pull`, `git push`, and `git status` natively within the GUI without opening a terminal.

### Modified Capabilities
None (this is the initialization phase).

## Impact

- Scaffolds the entire initial Tauri/Svelte workspace.
- Dictates the frontend/backend communication flow (IPC).
- Institutes the prerequisite checks (Git CLI present) on app boot.
