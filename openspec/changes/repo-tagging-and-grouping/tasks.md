## 1. SQLite Database Layer

- [x] 1.1 Add `rusqlite` with `bundled` feature to `src-tauri/Cargo.toml`
- [x] 1.2 Create `src-tauri/src/services/database.rs` — initialise SQLite connection, create `tags` and `repo_tags` tables with `CREATE TABLE IF NOT EXISTS`, enable WAL mode
- [x] 1.3 Create database helper function `get_db_path()` that stores `harbor.db` in the Tauri app data directory
- [x] 1.4 Manage the database connection as Tauri state (`DbPool` wrapper with `Mutex<Connection>`) registered in `lib.rs`
- [x] 1.5 Export database module in `src-tauri/src/services/mod.rs`

## 2. Tag CRUD Backend Commands

- [x] 2.1 Create `src-tauri/src/controllers/tags.rs` with Tauri command `list_tags` — returns all tags with repo counts via `LEFT JOIN` query
- [x] 2.2 Add Tauri command `create_tag(name, color)` — inserts into `tags` table, returns the new tag; validate uniqueness and name length (≤32 chars)
- [x] 2.3 Add Tauri command `rename_tag(id, new_name)` — updates tag name; validate uniqueness
- [x] 2.4 Add Tauri command `delete_tag(id)` — deletes from `tags` table (cascade deletes `repo_tags` entries)
- [x] 2.5 Add Tauri command `assign_tag(repo_path, tag_id)` — inserts into `repo_tags` (INSERT OR IGNORE for idempotency)
- [x] 2.6 Add Tauri command `remove_tag(repo_path, tag_id)` — deletes specific `repo_tags` entry
- [x] 2.7 Add Tauri command `get_repo_tags(repo_path)` — returns all tags for a specific repo
- [x] 2.8 Register all new tag commands in `lib.rs` invoke handler
- [x] 2.9 Export tags controller in `src-tauri/src/controllers/mod.rs`

## 3. Tag Integration with Scan & Cache

- [x] 3.1 Add `tags: Vec<String>` field to `RepoMetadata` struct in `src-tauri/src/models/repo.rs`
- [x] 3.2 Create `cleanup_orphaned_tags(valid_paths, db)` function in `database.rs` — deletes `repo_tags` entries where `repo_path` is not in the valid set
- [x] 3.3 Create `batch_fetch_repo_tags(db)` function in `database.rs` — returns a `HashMap<String, Vec<String>>` mapping repo paths to tag name lists
- [x] 3.4 Modify `refresh_repos` in `controllers/repo.rs` — after cache update, call `cleanup_orphaned_tags` with current cache keys and then merge batch-fetched tags into each `RepoMetadata` in the cache
- [x] 3.5 Modify `list_repos` to ensure tags are populated when reading from cache

## 4. Tag Model & Shared Types

- [x] 4.1 Create `Tag` struct in `src-tauri/src/models/repo.rs` (or a new `tag.rs`) with `id: i64`, `name: String`, `color: String`, `repo_count: i64`; derive `Serialize`, `Deserialize`
- [x] 4.2 Define curated colour palette constant (12 distinct hex colours) for frontend and backend validation

## 5. Frontend Tag Store

- [x] 5.1 Create `src/lib/stores/tagStore.ts` with Svelte writable stores: `allTags`, `selectedTagIds`
- [x] 5.2 Add `loadTags()` helper that invokes `list_tags` and updates the store
- [x] 5.3 Add `createTag()`, `renameTag()`, `deleteTag()`, `assignTag()`, `removeTag()` wrapper functions that call Tauri commands and then refresh the store
- [x] 5.4 Subscribe to `scan-completed` event to auto-refresh tags after scan

## 6. Sidebar Tag Navigation UI

- [x] 6.1 Add a collapsible "Tags" section in `+layout.svelte` between the nav links and the status footer
- [x] 6.2 Design the section header with "Tags" label, repo count badge, collapse/expand chevron with smooth rotation animation, and "+" create button
- [x] 6.3 Render the tag list — each item shows a colour dot, tag name, and repo count badge. Apply glassmorphism styling consistent with the existing sidebar design
- [x] 6.4 Implement click-to-filter: clicking a tag toggles it in `selectedTagIds` store; selected tags get highlighted with `bg-primary/20` and ring accent
- [x] 6.5 Implement "All" button/link at the top of tag list to clear tag filter
- [x] 6.6 Implement the inline create popover: text input + colour palette grid + confirm button. Opens on "+" click
- [x] 6.7 Implement right-click context menu on each tag with "Rename" and "Delete" options
- [x] 6.8 Implement inline rename (input replaces tag name on context menu "Rename")
- [x] 6.9 Implement delete with confirmation toast/prompt
- [x] 6.10 Add smooth enter/exit animations for tags appearing/disappearing in the list
- [x] 6.11 Handle empty state — show "Create your first tag" prompt with a subtle icon

## 7. Repo Card Tag Display & Assignment

- [x] 7.1 Display tag badges on repo cards (grid view) — show up to 3 tags as small colour-coded badges below the language badges, with "+N" overflow
- [x] 7.2 Display tag badges on repo list items (list view) — inline small badges after the language badges
- [x] 7.3 Create a tag assignment popover component — triggered by a "Tag" button or icon on the repo card. Shows all available tags as a checklist with checkmarks for currently assigned tags
- [x] 7.4 Wire popover toggle/assign/unassign actions to tagStore functions
- [x] 7.5 Ensure tag badges in the card link visually to sidebar colours (same colour dot)

## 8. Filter Integration

- [x] 8.1 In `+page.svelte`, subscribe to `selectedTagIds` from the tag store
- [x] 8.2 Extend the `filteredRepos` derived block to also filter by selected tags — a repo passes if it has ANY of the selected tags (OR logic), or if no tags are selected
- [x] 8.3 Ensure tag filter works in combination with search query and language filter

## 9. Group-By Framework/Language

- [x] 9.1 Add a "Group By" dropdown/toggle button in the toolbar (next to the grid/list toggle). Options: "None" (default), "Language"
- [x] 9.2 Create a `groupByMode` state variable (persisted to config via `set_config`/`get_config`)
- [x] 9.3 Create a `groupedRepos` derived block that groups `filteredRepos` by primary language (highest file count). Repos with no languages go under "Other". Sort groups alphabetically
- [x] 9.4 Create `GroupHeader.svelte` component — sticky section header with group name, repo count, and collapse/expand chevron. Styled with glassmorphism matching existing cards
- [x] 9.5 Implement collapse/expand state per group using a `collapsedGroups` Set state variable
- [x] 9.6 Render the grouped layout: when `groupByMode === 'language'`, iterate `groupedRepos` and render `GroupHeader` + repo cards/rows within each section. Support both grid and list view modes
- [x] 9.7 Add smooth slide animation for collapse/expand transitions
- [x] 9.8 Implement sticky header behaviour for group headers during scroll
- [x] 9.9 Hide groups that have zero repos after filtering

## 10. Config Persistence for New Preferences

- [x] 10.1 Extend `AppConfig` struct in `src-tauri/src/config/mod.rs` with `group_by_mode: Option<String>` field (default `None`)
- [x] 10.2 Load and apply `group_by_mode` preference on page mount
- [x] 10.3 Save `group_by_mode` preference when user changes it

## 11. Polish & Testing

- [x] 11.1 Verify tag CRUD operations work end-to-end (create, assign, rename, delete, cleanup)
- [x] 11.2 Verify sidebar tag filtering works with search and language filters simultaneously
- [x] 11.3 Verify grouped view renders correctly in both grid and list modes
- [x] 11.4 Verify collapse/expand animations are smooth and sticky headers work
- [x] 11.5 Verify tag data survives app restart (SQLite persistence)
- [x] 11.6 Verify orphan cleanup runs when watched folders change
- [x] 11.7 Run `cargo build` and `npm run build` to confirm no compilation errors
