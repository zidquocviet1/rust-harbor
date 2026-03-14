## 1. Backend Data & IPC

- [x] 1.1 Update the `AppConfig` struct in Rust to include new fields: `exclusion_patterns`, `max_depth`, `git_path`, and `auto_refresh`.
- [x] 1.2 Implement a `get_workspace_insights` Tauri command that returns repository counts and scan health for each watched folder.
- [x] 1.3 Update the `set_config` handler to validate and persist the enhanced configuration structure.
- [x] 1.4 Implement a `verify_git_path` command that checks binary existence and returns `--version` output.

## 2. Frontend Architecture & Layout

- [x] 2.1 Refactor `src/routes/settings/+page.svelte` to use a tab-based navigation system for settings categories.
- [x] 2.2 Create a shared `SettingsLayout` component with premium glassmorphism styling and sidebar navigation.
- [x] 2.3 Implement the `General` tab for managing watched folders with real-time insights (repo counts).
- [x] 2.4 Implement the `Performance` tab with UI for exclusion globs and recursion depth sliders.
- [x] 2.5 Implement the `Git Environment` tab with path input and a verification action.

## 3. Enhanced Scanning Engine

- [x] 3.1 Integrate glob-based filtering into the Rust directory traversal logic.
- [x] 3.2 Implement the recursion depth constraint in the backend scanner.
- [x] 3.3 Ensure the scanner respects the `auto_refresh` interval settings.
- [x] 3.4 Add diagnostic logging to capture permission errors during scans for the insight report.

## 4. UI/UX Polishing

- [x] 4.1 Apply global animation tokens to settings tab transitions.
- [x] 4.2 Enhance folder cards with hover effects, icons, and health indicators.
- [x] 4.3 Add a "Privacy & Security" info section that matches the updated aesthetic.
- [x] 4.4 Verify responsiveness across different window sizes.
