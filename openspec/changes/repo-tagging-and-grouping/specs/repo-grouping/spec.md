## ADDED Requirements

### Requirement: User can toggle group-by mode
The system SHALL provide a "Group By" control in the repository list toolbar that allows the user to switch between flat view (default) and grouped view. The grouped view SHALL group repositories by their primary detected programming language.

#### Scenario: Enable group-by language
- **WHEN** user clicks the "Group By" toggle and selects "Language"
- **THEN** the repository list reorganises into grouped sections, one per detected language, with repos sorted within each group by last modified time

#### Scenario: Disable group-by (return to flat view)
- **WHEN** user clicks the "Group By" toggle and selects "None" (or deselects the current grouping)
- **THEN** the repository list returns to the flat list/grid view sorted by last modified time

### Requirement: Primary language detection for grouping
The system SHALL determine each repository's "primary language" as the language with the highest file count in the `languages` HashMap. If the HashMap is empty, the repo SHALL be grouped under "Other".

#### Scenario: Repo with multiple languages
- **WHEN** a repo has languages `{Rust: 45, TypeScript: 12, HTML: 5}`
- **THEN** the repo is grouped under "Rust"

#### Scenario: Repo with no detected languages
- **WHEN** a repo has an empty languages HashMap
- **THEN** the repo is grouped under "Other"

#### Scenario: Tie in language count
- **WHEN** a repo has languages `{Python: 10, JavaScript: 10}`
- **THEN** the repo is grouped under the first language alphabetically ("JavaScript")

### Requirement: Grouped sections are collapsible
Each language/framework group section SHALL be independently collapsible and expandable. The section header MUST display the group name, repository count, and a collapse/expand chevron indicator.

#### Scenario: Collapse a group
- **WHEN** user clicks the header of the "Rust" group section
- **THEN** the repos within that group are hidden with a smooth slide-up animation, and the chevron rotates to indicate collapsed state

#### Scenario: Expand a collapsed group
- **WHEN** user clicks the header of a collapsed group
- **THEN** the repos within that group are revealed with a smooth slide-down animation

#### Scenario: All groups collapsed
- **WHEN** user collapses all groups
- **THEN** only group headers are visible, showing name and count for each

### Requirement: Group sections have sticky headers
Each group header SHALL be sticky-positioned so that it remains visible at the top of the viewport while scrolling through that group's repos.

#### Scenario: Scrolling within a large group
- **WHEN** user scrolls through a "JavaScript" group with 20 repos
- **THEN** the "JavaScript" group header remains pinned at the top of the scroll container until the next group's header reaches the top

### Requirement: Group-by preference is persisted
The system SHALL persist the user's group-by preference (enabled/disabled, and which grouping dimension) so it survives app restarts.

#### Scenario: User enables group-by and restarts
- **WHEN** user enables group-by language, closes and reopens the app
- **THEN** the repo list is displayed in grouped mode without requiring the user to re-enable it

### Requirement: Grouping works with both grid and list view modes
The grouped view SHALL render correctly in both grid and list view modes. In grid mode, each group's repos are displayed in a grid within the group container. In list mode, each group's repos are displayed as list rows within the group container.

#### Scenario: Grouped grid view
- **WHEN** group-by is enabled and view mode is "grid"
- **THEN** each group section contains a grid of repo cards

#### Scenario: Grouped list view
- **WHEN** group-by is enabled and view mode is "list"
- **THEN** each group section contains a list of repo rows

### Requirement: Grouping respects active filters
The grouped view SHALL respect all active filters (search query, language filter, tag filter). Groups with zero repos after filtering SHALL be hidden.

#### Scenario: Search within grouped view
- **WHEN** user has group-by enabled and types a search query
- **THEN** only groups containing matching repos are shown; non-matching groups are hidden

#### Scenario: Tag filter with grouping
- **WHEN** user has tag "work" selected and group-by enabled
- **THEN** only repos tagged "work" are shown, grouped by language; empty groups are hidden
