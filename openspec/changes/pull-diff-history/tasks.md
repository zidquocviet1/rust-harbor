## 1. Database Schema & Persistence Layer

- [x] 1.1 Add `pull_history` table to `database.rs` (`CREATE TABLE IF NOT EXISTS`) with columns: `id`, `repo_path`, `repo_name`, `branch`, `pulled_at`, `commit_before`, `commit_after`, `files_changed_count`
- [x] 1.2 Add `pull_history_files` table to `database.rs` with columns: `id`, `pull_id` (FK to pull_history), `file_path`, `change_type`, `additions`, `deletions`, `diff_content`
- [x] 1.3 Add `insert_pull_history` function to `database.rs` that inserts a pull record and all its file records in a single SQLite transaction
- [x] 1.4 Add `get_pull_history` query function to `database.rs` (returns list rows with summary fields, ordered by `pulled_at` DESC, supports optional `repo_path` filter)
- [x] 1.5 Add `get_pull_history_detail` query function to `database.rs` (returns full `pull_history_files` rows for a given `pull_id`)
- [x] 1.6 Add `delete_pull_history_entry` function to `database.rs` (deletes `pull_history` row + cascades to `pull_history_files`)
- [x] 1.7 Add `clear_pull_history` function to `database.rs` (deletes all rows from both tables)
- [x] 1.8 Enable `ON DELETE CASCADE` on `pull_history_files.pull_id` FK and verify PRAGMA foreign_keys is on

## 2. Data Models (Rust)

- [x] 2.1 Create `src-tauri/src/models/pull_history.rs` with structs: `PullHistoryEntry` (list view), `PullHistoryDetail` (with files), `PullHistoryFile`, `PullResult` (returned from `git_pull` command)
- [x] 2.2 Derive `serde::Serialize`, `serde::Deserialize`, and `Clone` for all model structs
- [x] 2.3 Add `pub mod pull_history;` to `src-tauri/src/models/mod.rs`

## 3. Enhanced git_pull Command (Rust)

- [x] 3.1 In `controllers/repo.rs`, before calling `git pull`, run `git rev-parse HEAD` to capture `commit_before` SHA
- [x] 3.2 After successful pull, run `git rev-parse HEAD` to capture `commit_after` SHA
- [x] 3.3 If `commit_before == commit_after`, return `PullResult { output, history_id: None }` (already up to date, skip history)
- [x] 3.4 Run `git diff <commit_before>..<commit_after> --numstat` to get per-file stats; parse each line into `(file_path, additions, deletions, change_type)`
- [x] 3.5 Run `git diff <commit_before>..<commit_after> --unified=3` to get full unified diff; split into per-file sections by parsing `diff --git` headers
- [x] 3.6 Handle binary files: numstat shows `-\t-` for binary; set `additions=0, deletions=0, diff_content="[binary file]"`
- [x] 3.7 Cap `diff_content` at 500KB per file; append `"\n[diff truncated]"` if exceeded
- [x] 3.8 Determine `change_type` from numstat output: new file → "added", deleted file → "deleted", rename → "renamed", else → "modified"
- [x] 3.9 Call `db.insert_pull_history(...)` with assembled data inside a try block; log warning on failure but do not fail the pull return
- [x] 3.10 Change `git_pull` Tauri command return type from `Result<String>` to `Result<PullResult>` and update the frontend call sites

## 4. New Tauri Commands for Pull History (Rust)

- [x] 4.1 Create `src-tauri/src/controllers/pull_history.rs` with command `get_pull_history(repo_path: Option<String>) -> Result<Vec<PullHistoryEntry>>`
- [x] 4.2 Add command `get_pull_history_detail(pull_id: i64) -> Result<PullHistoryDetail>` to `pull_history.rs`
- [x] 4.3 Add command `delete_pull_history_entry(pull_id: i64) -> Result<()>` to `pull_history.rs`
- [x] 4.4 Add command `delete_pull_history_entries(pull_ids: Vec<i64>) -> Result<()>` to `pull_history.rs` (bulk delete)
- [x] 4.5 Add command `clear_pull_history() -> Result<()>` to `pull_history.rs`
- [x] 4.6 Add `pub mod pull_history;` to `src-tauri/src/controllers/mod.rs`
- [x] 4.7 Register all new commands in `src-tauri/src/lib.rs` `invoke_handler`

## 5. UI Design — Pull History Page (invoke ui-ux-pro-max skill)

- [x] 5.1 Design the `/pull-history` page layout: header with title, entry count badge, filter controls (repo dropdown + date range), "Clear All" button, and scrollable list area
- [x] 5.2 Design the `PullHistoryCard` component: glassmorphism card matching existing repo card style, showing repo name, branch badge, timestamp, files changed count, commit range SHA pills (before → after arrow)
- [x] 5.3 Design the expanded detail panel within the card: two-column commit info (before/after), file list with change-type color badges (green=added, blue=modified, red=deleted, yellow=renamed), additions/deletions counters
- [x] 5.4 Design the `FileDiffViewer` component: line-numbered unified diff, green highlight + `+` prefix for additions, red highlight + `-` prefix for deletions, muted gray for context lines, monospace font (JetBrains Mono), collapse/expand toggle
- [x] 5.5 Design the empty state: centered illustration (GitPullRequest icon large), text "No pull history yet", subtext instructions
- [x] 5.6 Design the sidebar "Pull History" nav item with GitPullRequest icon, label, and numeric badge (indigo pill, same style as tag count badges)

## 6. Frontend Store & API Layer

- [x] 6.1 Create `src/lib/stores/pullHistoryStore.ts` with writable store for `PullHistoryEntry[]` and derived stores for filter state (selectedRepo, dateRange)
- [x] 6.2 Add `loadPullHistory(repoPath?: string)` function that calls `invoke('get_pull_history', ...)` and populates the store
- [x] 6.3 Add `loadPullHistoryDetail(pullId: number)` that calls `invoke('get_pull_history_detail', ...)`
- [x] 6.4 Add `deletePullHistoryEntry(pullId: number)` and `deleteMultipleEntries(pullIds: number[])` store actions
- [x] 6.5 Add `clearPullHistory()` store action
- [x] 6.6 Add `unreadCount` writable store; increment on `history_id` present in pull result from repo cards; reset to 0 on visiting `/pull-history`

## 7. Frontend Components

- [x] 7.1 Create `src/lib/components/pull-history/PullHistoryCard.svelte` — list item card with expand/collapse, delete button (with confirmation), checkbox for multi-select
- [x] 7.2 Create `src/lib/components/pull-history/CommitInfo.svelte` — shows before/after SHA, commit message, author, date in a two-column layout
- [x] 7.3 Create `src/lib/components/pull-history/FileList.svelte` — renders the list of changed files with type badge and +/- counts; each file row is clickable to expand diff
- [x] 7.4 Create `src/lib/components/pull-history/FileDiffViewer.svelte` — parses unified diff string into lines, renders with line numbers and highlight colors using JetBrains Mono
- [x] 7.5 Create `src/lib/components/pull-history/EmptyState.svelte` — empty state illustration + text
- [x] 7.6 Create `src/lib/components/pull-history/HistoryFilters.svelte` — repo dropdown and date range picker

## 8. Pull History Page Route

- [x] 8.1 Create `src/routes/pull-history/+page.svelte` — imports store, renders filter bar + card list, handles multi-select state and bulk-delete toolbar
- [x] 8.2 On page mount, call `loadPullHistory()` and reset `unreadCount` to 0
- [x] 8.3 Implement repo filter: derive filtered list from store based on `selectedRepo`
- [x] 8.4 Implement date filter: filter entries by `pulled_at` within selected range
- [x] 8.5 Implement "Clear All History" button with confirmation dialog (using bits-ui Dialog)
- [x] 8.6 Implement multi-select mode toggle; show "Delete Selected (N)" toolbar when items selected

## 9. Sidebar Navigation Update

- [x] 9.1 In `src/routes/+layout.svelte`, add "Pull History" nav item between Repository List and Settings with `GitPullRequest` icon from lucide-svelte
- [x] 9.2 Import `unreadCount` from `pullHistoryStore` and render a badge on the sidebar item when count > 0
- [x] 9.3 On successful pull from any repo card, check returned `history_id`; if present, increment `unreadCount`

## 10. Integration & Polish

- [x] 10.1 Update repo card pull action to handle new `PullResult` return type (extract `output` for toast, use `history_id` to update badge)
- [x] 10.2 Add toast notification after pull with link "View diff →" that navigates to `/pull-history` when `history_id` is present
- [x] 10.3 Verify the `pull_history` and `pull_history_files` tables are created on first app launch (test with fresh DB)
- [x] 10.4 Test pull with a repo that has large diffs to verify 500KB cap and truncation message works
- [x] 10.5 Test repository removal from watchlist and verify history entries are still visible on `/pull-history`
- [x] 10.6 Verify the unread count badge clears on visiting the Pull History page
