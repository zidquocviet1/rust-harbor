## Context

Rust Harbor users need a way to open repositories in their favorite IDEs/Editors directly from the app. Since different users have different tools installed, the app must dynamically detect and show only the relevant options. This requires a robust detection mechanism for macOS, Windows, and Linux.

## Goals / Non-Goals

**Goals:**
- Detect common IDEs/Editors installed on macOS, Windows, and Linux.
- Provide a clean UI to select an editor for a repository.
- Execute the correct system command to open the folder in the selected tool.
- Ensure the detection logic is fast and doesn't block the UI.

**Non-Goals:**
- Supporting every single editor in existence (start with the top 10-15).
- Supporting custom user-defined commands for editors (can be a future feature).
- Deep integration with IDEs (e.g., opening specific files within the IDE from Rust Harbor).

## Decisions

### 1. Detection Mechanism
**Choice**: Use platform-specific detection methods (bundle IDs for macOS, Registry/PATH for Windows, PATH/Desktop files for Linux).
- **Rationale**: 
  - **macOS**: `open -b <bundle_id> <path>` is a reliable way to open a folder in a specific app.
  - **Windows**: Check Registry keys (e.g., `HKEY_CLASSES_ROOT` or `Software\Microsoft\Windows\CurrentVersion\App Paths`) and common installation directories.
  - **Linux**: Check for binary existence in `PATH` and scan `/usr/share/applications` for `.desktop` files.
- **Alternatives**:
  - `mdfind`: Very accurate but can be slow if indexed metadata is lagging.
  - checking `/Applications/`: Won't find apps installed in user home or other locations.
  - `which <cli>`: Only works if the user has installed the CLI tool in their PATH, which many don't.

### 2. Editor Metadata
**Choice**: Hardcode a list of "Well-known Editors" in the Rust backend.
- **Rationale**: Keeps the logic simple and reliable. Each entry will have a name, bundle ID, and a fallback CLI name.

### 3. Architecture
- **Backend (Rust)**:
  - `EditorInfo` model with `id`, `name`, `icon`.
  - `editor_service`: Logic to filter the "Well-known" list down to what's actually on disk.
  - `editor_controller`: Exposure via Tauri `invoke`.
- **Frontend (Svelte)**:
  - Repository list context menu addition.
  - Repository details panel "Open In" dropdown button.

## Risks / Trade-offs

- **[Risk] Detection Speed** → [Mitigation] Cache the list of installed editors at startup and refresh only on demand.
- **[Risk] Path quoting** → [Mitigation] Use Tauri's `Command` API correctly to handle spaces in paths.
- **[Risk] Sandboxing** → [Mitigation] Since this is a Tauri app, we need to ensure the `opener` or `process` scope allows executing these commands if strict CSP/sandboxing is enabled.
