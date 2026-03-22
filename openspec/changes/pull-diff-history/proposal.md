## Why

Rust Harbor users perform git pulls on repositories but have no way to review what changed — the operation succeeds silently with no record of which files were affected or what commits were introduced. A persistent pull diff history gives users full visibility into every pull's impact across all their repositories, even after a repo is removed from the watchlist.

## What Changes

- **git_pull command enhanced** to capture HEAD SHA before and after the pull, compute the diff, and persist the result to SQLite
- **New SQLite tables** (`pull_history`, `pull_history_files`) store pull records and per-file diffs permanently
- **New Tauri commands** expose pull history data to the frontend: list, detail, delete, and clear
- **New sidebar navigation item** "Pull History" added between Repository List and Settings
- **New `/pull-history` route** with a list view of all pull events across all repositories
- **Detail view** per pull entry showing before/after commit metadata and an expandable file diff viewer with syntax-highlighted, line-by-line diff output

## Capabilities

### New Capabilities

- `pull-history-tracking`: Capture and persist git pull events — before/after commit SHAs, branch, timestamp, repo identity — into SQLite on every successful pull operation
- `pull-history-viewer`: UI page listing all pull history entries with filtering, detail expansion, and per-file diff viewing
- `pull-history-management`: User controls to delete individual entries or clear all history

### Modified Capabilities

- `git-pull-operation`: The existing pull command gains pre/post SHA capture and diff persistence as a side effect of each successful pull

## Impact

- **Backend**: `src-tauri/src/controllers/repo.rs` (git_pull enhancement), `src-tauri/src/services/database.rs` (new tables + queries), new `src-tauri/src/controllers/pull_history.rs`, `src-tauri/src/models/pull_history.rs`, `src-tauri/src/lib.rs` (command registration)
- **Frontend**: `src/routes/+layout.svelte` (sidebar nav), new `src/routes/pull-history/+page.svelte`, new store `src/lib/stores/pullHistoryStore.ts`, new components under `src/lib/components/pull-history/`
- **Dependencies**: No new dependencies needed — `git2` crate already available for diff parsing; `rusqlite` already used for persistence; `highlight.js` already available for syntax highlighting
- **Database**: Additive schema migration — new tables only, no existing table changes
- **Storage**: Diff content stored as text in SQLite; large diffs may add DB size over time (manageable with the delete/clear history controls)
