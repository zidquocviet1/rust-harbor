## Context

Rust Harbor is a Tauri + Svelte desktop application that scans local watched folders for Git repositories and presents them in a grid/list dashboard. The current architecture follows a View (Svelte) → Controller (Rust Tauri commands) → Data Layer (filesystem + git2 interrogation) pattern. Repository metadata is cached in an in-memory `DashMap` (`RepoCache`), and user preferences (watched folders) are stored in a simple JSON config file.

Currently there is **no persistent database** — all repo data is derived dynamically from the filesystem at scan time. The sidebar has only two navigation items (Repository List, Watched Folders) with no dynamic content. Repo filtering is limited to fuzzy text search and language badge toggles.

The PRD explicitly calls for tagging/categorisation as a supportive feature and mentions `rusqlite` as a dependency. The languages HashMap already exists on `RepoMetadata` and can serve as the data source for framework/language grouping.

## Goals / Non-Goals

**Goals:**
- Introduce a lightweight SQLite database to persist tag data across sessions
- Allow users to create, assign, modify, and delete custom tags on repositories
- Display a dynamic, beautiful tag navigation section in the sidebar
- Auto-cleanup orphaned tag associations when repos disappear after a watched-folder update
- Enable group-by-language/framework view with collapsible accordion sections
- Persist user preferences (group-by mode, collapsed states) in config
- Maintain the existing premium aesthetic — all new UI must match the dark glassmorphism design system

**Non-Goals:**
- Auto-tagging or AI-powered tag suggestions (future enhancement)
- Tag import/export or sync across devices
- Nested/hierarchical tags — flat tags only for v1
- Group-by-tag (only group-by-language/framework in this change)
- Changing the existing search or language filter behaviour

## Decisions

### Decision 1: SQLite via `rusqlite` with bundled feature

**Choice:** Use `rusqlite` with `bundled` feature for tag persistence.

**Rationale:** The PRD explicitly lists `rusqlite` as a dependency. SQLite is ideal because:
- Zero-configuration — the DB file is created automatically in the app data directory
- No external server needed; perfect for desktop apps
- ACID-compliant transactions for data integrity
- The `bundled` feature compiles SQLite from source, avoiding system dependency issues across macOS/Windows/Linux

**Alternatives considered:**
- *JSON file* — Too brittle for relational data (tag ↔ repo mapping), no transaction support, risk of data corruption on concurrent writes.
- *sled (embedded key-value store)* — Over-engineered for simple relational lookups; weaker ecosystem.

### Decision 2: Database schema with separate `tags` and `repo_tags` tables

**Choice:** Two-table design:
```sql
CREATE TABLE tags (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  color TEXT NOT NULL DEFAULT '#6366f1'
);

CREATE TABLE repo_tags (
  repo_path TEXT NOT NULL,
  tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY (repo_path, tag_id)
);
```

**Rationale:** Using `repo_path` as the foreign key (not an auto-increment repo ID) avoids needing to maintain a separate repos table. Since repos are transient (they come and go as watched folders change), the path is the only stable identifier already used throughout the system (`RepoCache` keys on path).

**Alternatives considered:**
- *Single JSON field per repo in a repos table* — Harder to query "all repos with tag X", no referential integrity.
- *Separate `repos` table with integer ID* — Unnecessary indirection; paths are already the cache keys and are unique. Would need constant syncing of the repos table with the cache.

### Decision 3: Tag cleanup strategy on watched-folder update

**Choice:** After `refresh_repos` completes and the cache is updated, run a cleanup query:
```sql
DELETE FROM repo_tags WHERE repo_path NOT IN (?, ?, ...)
```
passing the current valid repo paths from the cache.

Tags themselves (in the `tags` table) are never auto-deleted — users may want to keep empty tags for future use. A "Delete tag" action exists for explicit removal.

**Rationale:** Cleanup at refresh time is the natural point — it's when the canonical repo list is rebuilt. Running it after cache update ensures we have the authoritative list of paths.

### Decision 4: Inject tags into `RepoMetadata` at scan time

**Choice:** Add `tags: Vec<String>` to the `RepoMetadata` struct. After the parallel Rayon scan builds the metadata vector, perform a single batch SQL query to fetch all tag assignments, then merge them into the metadata objects before inserting into the `RepoCache`.

**Rationale:** Doing a single DB query after scan is far more efficient than querying per-repo during the parallel scan (which would also need connection pooling). The current flow: scan → build metadata → insert cache. New flow: scan → build metadata → batch-fetch tags → merge → insert cache → cleanup orphans.

### Decision 5: Group-by uses primary language detection

**Choice:** Determine each repo's "primary language" as the language with the highest file count in `repo.languages`. If the map is empty, group under "Other". The grouping is computed on the frontend in a `$derived` reactive block.

**Rationale:** Frontend grouping is simpler, requires no backend changes for the grouping logic itself, and allows instant toggle without re-fetching data. The language data already exists in `RepoMetadata.languages`.

### Decision 6: Sidebar tag section with inline management

**Choice:** Add a collapsible "Tags" section in the sidebar (`+layout.svelte`) between the nav links and the status footer. Tags are fetched via a new `list_tags` Tauri command on mount. The section supports:
- Inline "+" button to create a new tag (name + colour picker)
- Right-click context menu for rename/delete
- Click to filter the repo list (communicated via Svelte store)

**Rationale:** Keeping tag management in the sidebar follows the common pattern of apps like Finder (macOS smart folders), Notion (workspace tags), and Bear (tag sidebar). It provides always-visible context without modal interruptions.

### Decision 7: State communication via Svelte writable stores

**Choice:** Create a shared `tagStore.ts` with:
- `selectedTags: Writable<string[]>` — currently active tag filters
- `allTags: Writable<Tag[]>` — full tag list from DB
- Helper functions to refresh tags from backend

The main page subscribes to `selectedTags` and filters the repo list accordingly.

**Rationale:** Svelte stores are the idiomatic way to share state between layout (sidebar) and page components without prop drilling. They are reactive and lightweight.

## Risks / Trade-offs

- **[Risk] SQLite file corruption on force-quit** → Mitigation: SQLite WAL mode handles this gracefully. Enable `PRAGMA journal_mode=WAL` on connection open.
- **[Risk] Large number of repos makes cleanup query slow** → Mitigation: The `IN (...)` clause with hundreds of paths is well within SQLite's capability. Index on `repo_tags.repo_path` ensures fast lookups.
- **[Risk] Tag colour conflicts (similar colours hard to distinguish)** → Mitigation: Provide a curated palette of 12 distinct colours rather than a free-form colour picker.
- **[Trade-off] Using repo_path as key means renaming/moving a repo loses its tags** → Accepted: This is consistent with the existing system where the cache also keys on path. A future enhancement could detect renames via repo identity (commit hash of first commit).
- **[Trade-off] Frontend grouping means no persistent group structure** → Accepted: The grouping is a view-layer concern. Persisting only the "is grouping enabled" preference is sufficient.
