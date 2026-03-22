## ADDED Requirements

### Requirement: Pull History navigation entry in sidebar
The system SHALL display a "Pull History" navigation item in the left sidebar between "Repository List" and "Settings".

#### Scenario: Sidebar displays Pull History item
- **WHEN** the application loads
- **THEN** the sidebar SHALL show a "Pull History" item with a `GitPullRequest` lucide icon

#### Scenario: Pull History item shows unread badge
- **WHEN** new pull history entries exist that were created after the user's last visit to the Pull History page
- **THEN** the sidebar item SHALL display a numeric badge with the count of new entries

#### Scenario: Navigating to Pull History
- **WHEN** the user clicks the "Pull History" sidebar item
- **THEN** the app SHALL navigate to `/pull-history` and clear the unread badge

### Requirement: Pull History list page shows all pull events
The system SHALL provide a `/pull-history` page listing all captured pull events across all repositories, ordered by most recent first.

#### Scenario: Page loads with history entries
- **WHEN** the user navigates to `/pull-history` and there are recorded pull events
- **THEN** the page SHALL display a list of pull history cards, each showing: repository name, branch name, pull timestamp (relative + absolute on hover), number of files changed, and short commit SHAs (before → after)

#### Scenario: Page loads with no history
- **WHEN** the user navigates to `/pull-history` and no pull events have been recorded
- **THEN** the page SHALL display an empty state with an illustration and the message "No pull history yet. Pull a repository to see file changes here."

#### Scenario: Filter by repository
- **WHEN** the user selects a repository name from the filter dropdown
- **THEN** the list SHALL show only pull history entries for that repository

#### Scenario: Filter by date range
- **WHEN** the user selects a date range from the date filter
- **THEN** the list SHALL show only pull history entries within the selected date range

### Requirement: Pull History detail view shows commit and file changes
The system SHALL allow users to expand a pull history entry to view detailed commit information and a list of all changed files.

#### Scenario: Expanding a pull history entry
- **WHEN** the user clicks on a pull history card
- **THEN** the card SHALL expand inline (or navigate to a detail panel) to show: full before/after commit SHAs, commit message for the after commit, author name and date, and a list of all changed files grouped by change type

#### Scenario: File list in detail view
- **WHEN** a pull history entry is expanded
- **THEN** each changed file SHALL be shown with: file path, change type badge (Added/Modified/Deleted/Renamed in distinct colors), additions count (+N in green), and deletions count (-N in red)

### Requirement: File diff viewer shows syntax-highlighted line-by-line diff
The system SHALL provide an expandable diff viewer for each changed file within a pull history detail view.

#### Scenario: Expanding a file diff
- **WHEN** the user clicks on a file row in the detail view
- **THEN** the file diff viewer SHALL expand to show the unified diff with: line numbers, addition lines highlighted in green with `+` prefix, deletion lines highlighted in red with `-` prefix, and context lines in muted gray

#### Scenario: Binary file diff
- **WHEN** a changed file is binary
- **THEN** the file row SHALL show "Binary file" label and SHALL NOT be expandable for diff viewing

#### Scenario: Collapsed by default
- **WHEN** a pull history entry is expanded for the first time
- **THEN** all file diffs SHALL be collapsed by default; the user must click each file to expand its diff
