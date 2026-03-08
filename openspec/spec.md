prd:
  document_information:
    title: Rust Harbor
    version: 1.0
    date: March 2, 2026
    author: Grok AI (Assisted by User Input)
    purpose: This PRD serves as a contract for AI agents to follow in developing a Rust-based desktop application for managing Git-linked projects. It outlines the core features, additional supportive functionalities, user requirements, and technical specifications to ensure the application helps developers efficiently organize, search, and interact with their Git repositories from various platforms like GitHub, GitLab, etc.
  product_overview:
    description: The Rust Harbor is a lightweight, cross-platform desktop application built in Rust designed for developers who manage multiple Git repositories pulled from diverse platforms. It provides a centralized interface to discover, organize, search, and manage local Git projects, reducing the time spent navigating file systems or using command-line tools. The app scans local directories for Git repositories, displays them in an intuitive UI, and enables quick actions like searching, opening, and basic Git operations.
    goals_and_objectives:
      primary_goal: Enable users to quickly locate and manage their local Git projects without manual directory traversal.
      secondary_goals:
        - Improve productivity by integrating search and basic Git commands.
        - Support multi-platform Git repositories (e.g., GitHub, GitLab, Bitbucket).
        - Provide extensibility for future features like remote integrations.
        - Ensure a secure, performant, and user-friendly experience on desktop environments.
    scope:
      in_scope:
        - Local Git repository scanning and management.
        - Search functionality across repository metadata.
        - Basic Git operations (e.g., status, pull, push).
        - UI for listing and interacting with repositories.
        - Additional features like tagging, favorites, and notifications.
      out_of_scope:
        - Full-fledged Git client (e.g., no advanced branching or merging UI).
        - Web or mobile versions.
        - Direct integration with cloud services beyond basic authentication for remotes.
  user_personas:
    primary_user:
      description: Software Developer (e.g., a full-stack engineer like the user, managing 50+ repositories from GitHub, GitLab, and self-hosted Git servers).
      needs: Quick search for projects by name, keyword, or remote; easy access to open in IDE or terminal; overview of repository status.
      pain_points: Scattered repositories in multiple local folders; forgetting project locations; manual Git commands for routine checks.
    secondary_user:
      description: Open-Source Contributor.
      needs: Ability to clone new repos quickly and add them to the manager; tag repositories for categorization (e.g., "work", "personal").
    assumptions: Users are familiar with Git basics and have Rust development environment for building/contributing if needed.
  functional_requirements:
    core_features:
      repository_scanning_and_discovery:
        - Watched Folders Page: A dedicated UI page allowing users to add or remove specific folders to be watched.
        - The app shall only scan and update `.git` repositories located within these explicitly selected watched folders.
        - Automatically bind listeners when a folder is added or removed, ensuring the main repository list page instantly refreshes to reflect the current watch state.
        - Automatically detect remote origins (e.g., GitHub, GitLab) by parsing Git config files.
        - Support recursive scanning with options to exclude certain paths.
        - Store user preferences (like watched folders and app settings) in a lightweight local config file (e.g., JSON or TOML). Repository states and metadata will be derived directly and dynamically from the file system, removing the need for a heavyweight local database.
      repository_listing_and_dashboard:
        - Display a list or grid view of all detected repositories, showing:
        - Repository name.
        - Local path.
        - Remote URL and platform (e.g., GitHub icon for github.com remotes).
        - Current branch.
        - Status indicators (e.g., clean, dirty, ahead/behind remote).
        - Sortable and filterable by name, last modified, platform, or status.
      search_functionality:
        - Provide a global search bar to query repositories by:
        - Name or partial name.
        - Keywords in README or description (fetched from Git or remote if available).
        - Remote URL or platform.
        - Custom tags added by user.
        - Support fuzzy search for approximate matches.
        - Results should update in real-time as the user types.
    additional_supportive_features:
      tagging_and_categorization:
        - Users can add custom tags to repositories (e.g., "rust-projects", "web-apps", "archived").
        - Filter and group repositories by tags in the dashboard.
        - Search integration with tags for faster retrieval.
      quick_actions:
        - From the repository list, users can:
        - Open the repository folder in file explorer.
        - Open in preferred IDE (configurable, e.g., VS Code, IntelliJ).
        - Interact with dedicated UI components (e.g., buttons, quick action menus) to perform Git commands directly: fetch, pull, push, and view status (with confirmation prompts). Terminal launching is strictly out of scope.
        - Batch actions for multiple repositories (e.g., pull all tagged "active").
      clone_new_repositories:
        - Integrated clone dialog: Input remote URL, select local path, and clone directly into the app's managed list.
        - Auto-detect platform and suggest authentication if needed.
      authentication_and_remote_integration:
        - Support OAuth or token-based authentication for GitHub, GitLab, etc., to fetch additional metadata (e.g., repo description, stars).
        - Cache remote data locally to minimize API calls.
        - Handle private repositories with secure credential storage (using OS keychain).
      notifications_and_alerts:
        - Desktop notifications for events like "repository needs pull" (based on periodic checks).
        - Alert for conflicts or errors during Git operations.
      backup_and_export:
        - Export repository list as JSON/CSV for backup.
        - Option to archive or remove repositories from the list (without deleting files).
      settings_and_customization:
        - User-configurable scan directories.
        - Theme options (light/dark mode).
        - Integration preferences (e.g., default IDE, Git binary path).
        - Auto-scan on app launch or manual refresh.
  non_functional_requirements:
    performance:
      - App startup time: < 2 seconds.
      - Scanning a directory with 100+ repositories: < 5 seconds.
      - Search response time: < 500ms for 500 repositories.
      - Memory usage: < 100MB under normal operation.
    security:
      - No storage of sensitive data (e.g., Git tokens) in plain text; use secure vaults like keyring crate.
      - Validate all user inputs to prevent injection attacks.
      - Ensure Git commands are executed safely without exposing the system.
    usability:
      - Intuitive UI/UX: Use Tauri as the core application framework, coupled with Svelte and shadcn-svelte on the frontend to ensure a highly polished, consistent, and beautiful UI component system.
      - Keyboard shortcuts for common actions (e.g., Ctrl+F for search).
      - Accessibility: Support screen readers, high contrast modes.
      - Localization: English as default; prepare for future multi-language support.
    reliability:
      - Handle edge cases: Non-Git directories, corrupted .git folders, network failures for remote fetches.
      - Error logging: Maintain a log file for debugging.
      - Auto-recovery: If database corrupts, offer re-scan option.
    compatibility:
      - Platforms: Windows, macOS, Linux.
      - Rust Version: Stable channel (e.g., 1.75+).
      - Dependencies: Minimize external crates; use `git2` for Git API interactions. Tauri, Svelte, and shadcn-svelte for the frontend UI. (No heavy SQL database required).
  technical_specifications:
    technology_stack:
      - Language: Rust.
      - Git Library: `git2` crate alongside direct system `git` CLI invocation where appropriate.
      - Persistent Storage: Local JSON/TOML configuration file for settings (Watched Folders, UI preferences).
      - UI Framework: Tauri + Svelte + shadcn-svelte.
      - Authentication: oauth2 or similar for Git providers.
    architecture:
      - Architecture Pattern: View (Svelte UI), Controller (Rust business logic via Tauri IPC), and Data Layer (Direct File System / Git interrogation).
      - Asynchronous operations for scanning and Git commands to keep UI responsive.
      - State Syncing & File Watchers: Implement file watchers on watched `.git` directories to trigger real-time UI updates (e.g., when a user makes pushes or commits). The UI must always accurately reflect the latest repo status.
      - Worker Periodic Polling: Utilize a background worker with periodic polling to supplement file watchers and ensure state remains synced. Performance/Security constraints: The polling interval must be heavily optimized (e.g., adaptive or configurable logic) to prevent CPU/IO spikes and avoid hitting OS file handle limits. Polling and file watching must strictly only operate on user-specified watched folders to ensure security and prevent unauthorized filesystem traversal.
    testing:
      - Unit tests for core functions (e.g., scanning, search).
      - Integration tests for UI interactions.
      - Coverage target: 80%.
  assumptions_and_constraints:
    system_prerequisites:
      - **Git CLI Check**: On application startup, the system MUST verify that the `git` CLI is installed and accessible in the system's `PATH`. If missing, the app should halt standard load and present a clear prerequisite error page guiding the user to install Git.
    assumptions:
      - Internet access for remote features (optional fallback to local-only mode).
      - Repositories are standard Git repos; no support for non-Git VCS.
    constraints:
      - Development timeline: Not specified; aim for MVP in 4-6 weeks with AI agents.
      - Budget: Open-source style; no paid APIs beyond free tiers.
      - Legal: Comply with Git providers' API terms (e.g., rate limits).
  dependencies_and_risks:
    dependencies:
      - External Crates: git2, rusqlite, iced/tauri, notify (for file watching), etc.
      - APIs: GitHub/GitLab REST APIs for metadata.
    risks:
      - Git command failures on user systems: Mitigate with error handling.
      - UI framework limitations: Choose based on cross-platform needs.
      - Data privacy: Ensure no unintended data sharing.
  appendix:
    wireframes_sketches: (To be added by AI agents; suggest a simple dashboard with search bar, list view, and sidebar for tags.)
    success_metrics: User satisfaction via feedback; track usage of search and quick actions.
    future_enhancements: Integration with Git hosting UIs, AI-based repo recommendations, cloud sync for repo lists.
  closing_note: This PRD is the guiding contract for development. Any deviations must be approved by the product owner.