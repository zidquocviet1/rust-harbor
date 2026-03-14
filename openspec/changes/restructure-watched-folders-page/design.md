## Context

The current Settings page in Rust Harbor is minimal, handling only a flat list of watched folders. The UI contains several disabled categories ("Performance", "Security", "Git Binary") that are non-functional placeholders. The codebase for settings is currently a single large Svelte component, which will become unmaintainable as we add more configuration options.

## Goals / Non-Goals

**Goals:**
- Implement a fully functional "Workspace Manager" with repository insights.
- Activate "Performance" and "Git Binary" settings with real backend integration.
- Apply a premium UI/UX design using glassmorphism, micro-animations, and better layout hierarchy.
- Componentize the settings page to support future scalability.
- Improve repository scanning performance via exclusion filters.

**Non-Goals:**
- Implementing the "Security" tab in this phase (remains a placeholder but with improved styling).
- Advanced Git config editing (e.g., gitconfig editing).
- Support for remote repository management in this change.

## Decisions

### 1. Robust Configuration Schema
We will transition the application configuration from a simple `watched_folders: Vec<String>` to a more comprehensive structure.
- **Rationale**: To support exclusion patterns, scan depth, and custom Git paths, the configuration must be a structured object.
- **Alternatives**: Using multiple small config files. *Rejected* because a single JSON/TOML file is easier to manage and keep in sync.

### 2. Component-Based Settings Layout
We will split the settings page into modular components: `GeneralSettings.svelte`, `PerformanceSettings.svelte`, `GitSettings.svelte`, and `SecuritySettings.svelte`.
- **Rationale**: Improves maintainability and allows for smooth transitions between tabs without reloading the entire page logic.
- **Alternatives**: Keeping everything in one file with large if/else blocks. *Rejected* as it leads to "prop-drilling" and bloated code.

### 3. Scan Filter Integration
The Rust backend will integrate glob-based filtering during the directory traversal.
- **Rationale**: Skipping `node_modules` or `target` at the folder level prevents excessive I/O and speeds up discovery.
- **Alternatives**: Scanning everything and then filtering the results. *Rejected* because it doesn't solve the performance bottleneck of traversing deep dependency trees.

### 4. UI/UX: Premium Glassmorphism & Animations
Use the `ui-ux-pro-max` design patterns:
- **Glassmorphism**: Use semi-transparent backdrops with blur filters for cards.
- **Micro-animations**: Implement `animate-in` and hover scales for interactive elements.
- **Visual Hierarchy**: Use `APP_BRANDING` tokens and `text-glow` for headings.

## Risks / Trade-offs

- **[Risk] Complex Glob Matching Performance** → **[Mitigation]** Pre-compile glob patterns at the start of a scan to minimize per-file processing time.
- **[Risk] Migration Path for Existing Configs** → **[Mitigation]** Implement robust deserialization in Rust that provides defaults for new fields if they are missing from older config files.
- **[Risk] OS Specific Git Paths** → **[Mitigation]** Default to "git" (system PATH) and only use custom paths if explicitly provided and verified.
