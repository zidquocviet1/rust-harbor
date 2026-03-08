## 1. Project Initialization

- [x] 1.1 Scaffold the initial user workspace via `create-tauri-app` using Svelte with TypeScript as the frontend structure.
- [x] 1.2 Initialize Svelte inside the Tauri project and install `shadcn-svelte` requirements.
- [x] 1.3 Update `src-tauri/Cargo.toml` to depend on `git2`, `notify`, `serde_json`, and other required libraries.
- [x] 1.4 Scaffold the Rust backend structure (Controller logic, Configuration handlers, Error definitions).

## 2. Configuration & Startup 

- [x] 2.1 Implement the pre-flight check in `main.rs`/Tauri setup to verify `git` exists within the system `PATH`.
- [x] 2.2 Create a blocking frontend modal or error page that displays if the pre-flight check fails.
- [x] 2.3 Implement the JSON/TOML reader/writer in Rust to persist and fetch the "Watched Folders" array on startup.
- [x] 2.4 Scaffold the `Watched Folders` Svelte configuration page to let users add or remove file paths via system dialog.

## 3. Data Layer & File Watching

- [x] 3.1 Implement a local Rust service to recursively scan explicitly "Watched Folders" for `.git` child directories.
- [x] 3.2 Implement the `notify` file watcher bound specifically to the identified `.git` directories, mapping system events to generic internal events.
- [x] 3.3 Set up Tauri IPC hooks allowing Svelte to subscribe to these file alteration events to trigger re-fetches.
- [x] 3.4 Wire the Svelte frontend dashboard to invoke the Rust scanner, returning the list of active repositories and mapping them to basic `shadcn` cards.

## 4. Git Integrations

- [x] 4.1 Write Rust endpoints using `git2` or `std::process::Command` to extract current branch, clean/dirty status, and remote URL for a given repository path.
- [x] 4.2 Populate the dashboard cards with this repository metadata via IPC.
- [x] 4.3 Write Tauri commands to invoke `git pull`, `git push`, and `git fetch` operations, streaming success or standard error strings back to the frontend.
- [x] 4.4 Build the UI Action Buttons on the Svelte dashboard to trigger these Git operations, wrapping them in `shadcn` toast notifications to display the result (success/error).

## 5. Review & Polish

- [x] 5.1 Perform a final walkthrough to ensure that repo cards update automatically when local files are changed.
