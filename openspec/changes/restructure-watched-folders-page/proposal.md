## Why

The current "Watched Folders" page is a basic implementation that lacks professional polish and contains several disabled categories ("Performance", "Security", "Git Binary") that detract from the user experience. As the application grows, users need a more robust "Workspace Manager" that provides better visibility into their scanned environments, performance controls to optimize large filesystems, and a UI that aligns with the premium aesthetic of the rest of the application.

## What Changes

- **Workspace Hub Overhaul**: Rebrand "Watched Folders" as a "Workspace Manager" with a more sophisticated layout using glassmorphism and enhanced visual hierarchy.
- **Folder Insights**: Display real-time statistics for each watched folder, such as the number of repositories found and last scan status.
- **Active Settings**: Implement the previously disabled "Performance" settings to allow users to configure scan depth and exclusion patterns.
- **Git Binary Configuration**: Enable the "Git Binary" settings to allow users to specify custom Git paths or verify their Git installation.
- **Premium UX**: Integrate advanced animations, hover effects, and a responsive sidebar structure using the `ui-ux-pro-max` design system.
- **Exclusion Filters**: Add the ability to specify global or per-folder ignore patterns (e.g., `node_modules`, `target`, `.venv`) to significantly improve scanning performance.

## Capabilities

### New Capabilities
- `workspace-insights`: Provide metadata and health status for each watched directory (repo count, scan success).
- `performance-optimizations`: Configuration for scan exclusion patterns, recursive depth limits, and auto-refresh intervals.
- `git-environment-management`: UI for Git binary verification and custom path configuration.

### Modified Capabilities
- `repository-scanning-and-discovery`: Update the scanning engine to respect exclusion patterns and depth limits defined in the new settings.

## Impact

- **Frontend**: Major restructure of `src/routes/settings/+page.svelte`. New components for workspace cards and setting toggles.
- **Backend (Tauri)**: Update the configuration data structures in Rust to store exclusion patterns, depth limits, and Git paths.
- **Scanning Logic**: Modify the repository discovery service to incorporate filter logic and performance constraints.
- **Storage**: Migrate or update the local config file schema to accommodate the new settings.
