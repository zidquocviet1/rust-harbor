## 1. Backend: Metadata & Connectivity

- [x] 1.1 Update `RepoMetadata` struct in `src-tauri/src/controllers/repo.rs` to include a weighted language map and repository description.
- [x] 1.2 Implement a new `analyze_languages` function in Rust to replace the single-extension detection.
- [x] 1.3 Implement `verify_remote_connectivity` using `git ls-remote` with a 5-second timeout in `src-tauri/src/controllers/repo.rs`.
- [x] 1.4 Update the `refresh_repos` background task to populate the new metadata fields and execute connectivity probes.

## 2. Frontend: Filter & Metadata UX

- [x] 2.1 Refactor the language filter bar in `+page.svelte` to sort languages by frequency and add a "+x" overflow dropdown.
- [x] 2.2 Update Repo Card (Grid View) to display the new multi-language icons and repository description.
- [x] 2.3 Implement hover tooltips/labels for all Git actions across Grid and List views.

## 2. Frontend: README Side-Panel

- [x] 3.1 Implement a sliding side-sheet component with entry/exit animations.
- [x] 3.2 Create a Tauri command to securely retrieve README content (caching result for better performance).
- [x] 3.3 Implement the three-mode toggle (Markdown, Plain Text, Unified) in the side-panel UI.
- [x] 3.4 Configure repo card selection to trigger the README side-panel correctly.

## 4. Operation & List View Enhancements

- [x] 4.1 Swap "Open Directory" and "Refresh" for "Fetch" and "Pull" (current branch) in the List View repository items.
- [x] 4.2 Restrict the `git_pull` command context to avoid multi-branch merge conflicts in the UI.
- [x] 4.3 Final end-to-end testing of the background scan sync and connectivity status logic.
