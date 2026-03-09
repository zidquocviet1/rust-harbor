## Why

Users managing dozens (or hundreds) of local Git repositories need a way to organise and categorise them beyond what the file-system hierarchy provides. Currently, Rust Harbor displays a flat list/grid of repositories with language-based filtering only. There is no mechanism to assign custom labels ("work", "personal", "archived", "hackathon") or to visually group repositories by their primary framework or language — both features that are critical for power users who manage a large number of repos.

## What Changes

- **Custom tagging system** — Users can create, assign, and remove free-form tags to any repository. Tags are persisted in a local SQLite database with a `tags` and `repo_tags` join table.
- **Tag list in sidebar** — The left navigation panel displays a dynamic, beautifully styled tag list with repo counts. Clicking a tag filters the repo list to that tag instantly. Tags are colour-coded and support inline creation/rename/delete.
- **Tag sync on watched-folder update** — When watched folders change (add/remove), orphaned tag associations (repos that no longer exist) are automatically cleaned up. New repos discovered get no tags by default.
- **Group-by framework/language** — A new "Group By" toggle in the toolbar allows users to group the repository list by detected primary language or framework. Each group is a collapsible/expandable section with a sticky header showing the group name, repo count, and collapse control.
- **Collapsible group UI** — Each language/framework group is rendered as an accordion section with smooth open/close animation, persistent expand/collapse state, and drag-friendly section headers.

## Capabilities

### New Capabilities
- `repo-tagging`: Create, assign, remove, and manage custom tags for repositories. Persist tag data in SQLite. Auto-cleanup orphaned tags on watched-folder refresh. Surface tags in repo cards and search.
- `sidebar-tag-navigation`: Display an interactive, filterable tag list in the sidebar. Support inline CRUD on tags. Filter repo list by selected tag(s).
- `repo-grouping`: Group repository list by primary language or framework. Render collapsible/expandable sections. Persist user's group-by preference.

### Modified Capabilities
_None — no existing spec-level behaviour is changing._

## Impact

- **Backend (Rust / Tauri)**
  - New SQLite database layer (via `rusqlite`) for tag storage — tables: `tags`, `repo_tags`.
  - New Tauri commands: `list_tags`, `create_tag`, `delete_tag`, `assign_tag`, `remove_tag`, `get_repo_tags`, `cleanup_orphaned_tags`.
  - `refresh_repos` extended to call tag cleanup after scan completes.
  - New `RepoMetadata` field: `tags: Vec<String>` populated from the database at scan time.
  - Primary language/framework detection logic added to `repo_service.rs`.

- **Frontend (Svelte)**
  - `+layout.svelte` sidebar extended with a collapsible "Tags" section showing tag list with counts.
  - `+page.svelte`: repo cards display assigned tags; new "Group By" toolbar control; repo list renders in grouped accordion layout when group mode is active; tag assignment dialog/popover on each repo card.
  - New Svelte components: `TagBadge`, `TagManager`, `GroupHeader`.

- **Dependencies**
  - `rusqlite` crate added to `src-tauri/Cargo.toml`.
  - No new frontend npm packages required (shadcn-svelte components already available).
