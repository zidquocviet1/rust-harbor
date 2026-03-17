## Why

Users often need to quickly transition from viewing a repository in Rust Harbor to editing its code. Currently, they have to manually navigate to the repository path in their file explorer or open their editor and then the project. Providing an "Open in Editor" feature directly within the app improves developer productivity and provides a more seamless workflow.

## What Changes

- Add a new "Open in Editor" menu or button in the repository details panel and/or repository list context menu.
- Support a wide range of popular Editors and IDEs (e.g., VS Code, Cursor, Zed, JetBrains IDEs, Sublime Text, etc.).
- Implement logic to detect which Editors/IDEs are installed on the user's system (macOS, Windows, and Linux).
- Only display the "Open in Editor" options for the tools that are actually installed and available.

## Capabilities

### New Capabilities
- `open-in-editor`: Allows users to open a repository's root directory in their preferred, installed IDE or Editor.

### Modified Capabilities
<!-- No existing capabilities found in openspec/specs/ -->

## Impact

- **UI**: New components and layout changes to the repository details panel and context menus.
- **Backend (Rust/Tauri)**: New Tauri commands to:
  1. Detect installed IDEs/Editors.
  2. Execute shell commands to open a path in a specific IDE/Editor.
- **Service/Logic**: Logic to map IDE names to their corresponding CLI commands or bundle identifiers.
