## Why

The current repository dashboard provides a good overview but lacks depth and interactive efficiency. Users managing a large number of repositories (40+) need better information density and quicker access to common tasks. 

Specifically:
- Primary language detection is too simplistic; projects are often multi-language.
- The filtering system becomes cluttered with many languages.
- Basic metadata like repository descriptions is missing from the card view.
- Viewing a project's README currently requires navigating away or opening a separate editor, which breaks the management workflow.
- Action icons can be ambiguous without text labels or tooltips.
- The list view doesn't prioritize the most frequent Git operations (fetch/pull).
- Remote connectivity status is currently a placeholder (URL check) rather than a real reachability check, leading to false "Connected" statuses for inaccessible repos.

## What Changes

### Frontend Improvements
- **Multi-language Display**: Grid items will now show icons and names for multiple languages found in the repo.
- **Advanced Filter Bar**: The language filter will be sorted by frequency. Overflows will be handled by a "+x" dropdown.
- **Metadata Injection**: Repository cards in grid view will now include a brief description.
- **Action Tooltips**: All action buttons will display their name/purpose on hover.
- **Action Swap (List View)**: In list view, the "Open" and "Refresh" buttons will be replaced with direct "Fetch" and "Pull" (current branch) buttons.
- **Integrated Preview**: Clicking a repository will open an animated side panel on the right highlighting the README content with three viewing modes: Plain Text, Markdown, and Unified.

### Backend Improvements
- **Deep Language Detection**: The Rust scanner will return a map of languages and their relative weights.
- **True Connectivity Check**: The `remote_reachable` logic will be upgraded to perform a lightweight probe (e.g., `git ls-remote` with a timeout) to ensure real-time connectivity status.
- **Metadata Extraction**: Enhance the scanner to pull descriptions (from `.git/description` or detected README headers).

## Capabilities

### New Capabilities
- `multi-language-analysis`: Weighted language detection across the entire repository and intelligent filter management.
- `readme-preview-panel`: Dynamic, animated side-sheet for README inspection with multiple rendering modes.

### Modified Capabilities
- `git-action-optimization`: Optimized list-view actions and realistic remote connectivity verification.

## Impact
- **Backend (`src-tauri/src/controllers/repo.rs`)**: Significant update to `RepoMetadata` struct and detection logic.
- **Frontend (`src/routes/+page.svelte`)**: Major UI overhaul for the filter bar, repo cards, and the new side-panel component.
- **Tauri Commands**: Potential new command for lightweight connectivity probing or fetching README content.
