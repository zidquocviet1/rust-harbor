## Context

Rust Harbor uses a Tauri 2 desktop architecture with a Svelte 5 frontend and a Rust backend. Git operations are executed via `std::process::Command` (not `git2`), and results are returned to the frontend as strings via Tauri commands. Persistence uses SQLite via `rusqlite` with a single WAL-mode connection protected by a `Mutex`. The app has no existing history, audit trail, or diff viewing capability.

The current `git_pull` command in `controllers/repo.rs` runs `git pull origin HEAD` and returns the raw stdout/stderr string — nothing is captured or stored.

## Goals / Non-Goals

**Goals:**
- Capture before/after HEAD SHAs on every successful pull and persist a structured record to SQLite
- Store per-file diff metadata (path, change type, additions, deletions) and raw unified diff text per pull event
- Expose pull history via new Tauri commands (list, detail, delete, clear)
- Add a `/pull-history` route in the frontend with a searchable, filterable list and an expandable file diff viewer
- Add a sidebar nav entry for Pull History that shows a badge count of unseen entries
- History persists even after a repository is removed from the watchlist

**Non-Goals:**
- Viewing git log or arbitrary commit ranges (only pulls are tracked)
- Editing, reverting, or applying diff patches from the UI
- Syncing history across machines or remote storage
- Tracking fetch, push, or manual commits — only `git pull` events
- Real-time diff streaming for large repos (diff is captured synchronously post-pull)

## Decisions

### 1. Diff captured via `git diff <before>..<after>` subprocess, not `git2`

**Decision**: After a successful pull, run `git diff <before_sha>..<after_sha> --unified=3` and `git diff <before_sha>..<after_sha> --numstat` as separate subprocess calls using the same `execute_git_command` helper already in use.

**Why over `git2` diff API**: The existing pull operation already uses subprocess. Mixing `git2` and subprocess adds complexity and linking overhead. The subprocess approach is simpler, consistent with the existing codebase pattern, and the diff output is the standard unified format that the frontend can parse directly.

**Alternative considered**: `git2::Repository::diff_tree_to_tree` — rejected due to added complexity and inconsistency with current approach.

### 2. Diff content stored as raw unified text in SQLite

**Decision**: Store the full unified diff string per file in `pull_history_files.diff_content TEXT`. Diff is captured at pull time and stored once.

**Why**: Querying the diff later would require re-running git, which may fail if the commits are no longer available (garbage collected) or the repo is unwatched. Storing the diff at capture time guarantees permanent availability.

**Trade-off**: Large diffs (binary files, generated files, large refactors) can inflate DB size. Mitigated by: skipping binary files (detected via `--numstat` showing `-` for binary), and the user-facing "clear history" control.

### 3. Separate `pull_history` and `pull_history_files` tables (no JSON blob)

**Decision**: Normalized schema — one row per pull in `pull_history`, one row per changed file in `pull_history_files`.

**Why over a single JSON column**: Enables efficient list queries (count files, filter by repo, order by date) without deserializing blobs. Keeps the list view fast — only `pull_history` is queried for the list; `pull_history_files` is fetched only on detail open.

### 4. `git_pull` command returns an enriched result struct instead of a plain string

**Decision**: Change the return type of `git_pull` Tauri command from `Result<String>` to `Result<PullResult>` where `PullResult` contains `{ output: String, history_id: Option<i64> }`.

**Why**: The frontend needs to know if a history entry was created (to update the badge count) without making a separate query. `history_id: None` means the pull produced no new commits (already up to date).

**Alternative**: Fire-and-forget history capture in background task — rejected because it adds race conditions and makes error surfacing harder.

### 5. Frontend diff rendering via string parsing (no external diff library)

**Decision**: Parse the unified diff string on the frontend in a Svelte component using a lightweight custom parser. Render additions in green, deletions in red, context lines in muted gray.

**Why**: No suitable existing diff renderer is already in the project dependencies. Adding a new npm package for this single feature is overkill. The unified diff format is well-defined and a simple line-by-line parser is ~50 lines of TypeScript.

**Alternative considered**: `diff2html` npm package — rejected to avoid new dependencies.

### 6. New sidebar nav item between Repository List and Settings

**Decision**: Add "Pull History" nav item in `+layout.svelte` with a `GitPullRequest` lucide icon and an optional numeric badge showing unread count (entries since last visit).

**Why between Repo List and Settings**: Pull History is a primary feature (not a configuration item), so it belongs in the main nav above Settings.

## Risks / Trade-offs

- **[Large diffs]** → Mitigation: Skip binary files; cap diff storage at 500KB per file; truncate with a note in UI
- **[Pull fails mid-capture]** → Mitigation: Wrap the entire capture-and-persist block in a transaction; if insert fails, log warning but do not fail the pull itself (history is best-effort)
- **[Before SHA = After SHA (already up to date)]** → Mitigation: Detect this case (empty diff output or same SHA) and skip history creation; return `history_id: None`
- **[Repo removed from watchlist]** → History rows are intentionally orphaned by design — `repo_path` is stored as plain text, no foreign key constraint to `repositories` table
- **[DB size growth]** → Mitigation: "Clear all history" control; future work could add auto-pruning after N entries or N days

## Migration Plan

1. On app startup, `database.rs` runs schema migrations — add `CREATE TABLE IF NOT EXISTS pull_history ...` and `CREATE TABLE IF NOT EXISTS pull_history_files ...`
2. No existing data is affected — purely additive schema change
3. Rollback: tables can be dropped without affecting any existing functionality

## Open Questions

- Should binary file diffs be stored as a placeholder row (e.g., `diff_content = "[binary file]"`) or skipped entirely? → Default: store a placeholder row so the file appears in the changed files list but with `0` additions/deletions and no expandable diff.
- Should there be a configurable max history retention count/age? → Out of scope for v1; user can manually clear.
