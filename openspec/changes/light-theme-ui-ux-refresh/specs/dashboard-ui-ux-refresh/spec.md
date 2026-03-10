## ADDED Requirements

### Requirement: Light-first dashboard shell refresh
The repository dashboard shell SHALL be refreshed to a light-first visual hierarchy with clearer separation between navigation, toolbar, content, and overlays.

#### Scenario: Layout clarity
- **WHEN** the user opens the main dashboard
- **THEN** sidebar, toolbar, and content regions are visually distinct and easier to scan than the previous dark-heavy presentation

#### Scenario: Overlay readability
- **WHEN** dialogs, context menus, or popovers are shown
- **THEN** overlay surfaces and text remain high-contrast and readable in light mode

### Requirement: Repository card and list readability refresh
Repository cards and list rows SHALL use improved typographic hierarchy, spacing, and visual emphasis to prioritize key repository information.

#### Scenario: Grid card scanning
- **WHEN** repositories are shown in grid view
- **THEN** repository name, branch, status, and key metadata can be identified quickly with clear visual priority

#### Scenario: List row scanning
- **WHEN** repositories are shown in list view
- **THEN** row density remains efficient while preserving readability and interaction clarity

### Requirement: Filter and control usability refresh
Search, language filters, tag filters, and grouping controls SHALL be visually and behaviorally consistent in light mode.

#### Scenario: Multi-filter clarity
- **WHEN** users apply combinations of search, language, and tag filters
- **THEN** active filter states are clearly indicated and remain distinguishable from inactive controls

#### Scenario: Control discoverability
- **WHEN** users interact with toolbar and filter controls
- **THEN** controls provide consistent hover/focus/active feedback and do not require guesswork

### Requirement: Tag management UX consistency in light mode
Tag sidebar, tag badges, and tag assignment popovers SHALL maintain visual consistency and readability after the refresh.

#### Scenario: Sidebar tag interactions
- **WHEN** users create, rename, delete, or select tags in the sidebar
- **THEN** interactions remain clear with consistent selected states, readable counts, and legible text

#### Scenario: Repo tag assignment
- **WHEN** users open a repository tag assignment popover
- **THEN** current selection state is immediately understandable and updates are reflected clearly in the UI

### Requirement: Functional behavior parity
The UI refresh SHALL preserve existing repository management functionality and command behavior.

#### Scenario: Unchanged workflows
- **WHEN** users perform existing operations (refresh, open folder, git actions, filtering, preview, tagging)
- **THEN** workflow behavior remains functionally equivalent while visual UX is improved
