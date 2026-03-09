## ADDED Requirements

### Requirement: Sidebar displays a dynamic tag list
The sidebar (left navigation panel) SHALL display a "Tags" section that lists all user-created tags. Each tag entry MUST show the tag name, its assigned colour indicator, and the count of repositories assigned to it.

#### Scenario: Tags exist
- **WHEN** the user has created 3 tags ("work" with 5 repos, "personal" with 2 repos, "archived" with 0 repos)
- **THEN** the sidebar Tags section displays all 3 tags with their respective counts

#### Scenario: No tags exist
- **WHEN** the user has not created any tags
- **THEN** the sidebar Tags section shows an empty state with a prompt "Create your first tag" and a "+" button

### Requirement: Sidebar tag section is collapsible
The Tags section in the sidebar SHALL be collapsible/expandable. The collapse state MUST be persisted so it survives page navigation.

#### Scenario: Collapse the tag section
- **WHEN** user clicks the collapse toggle on the Tags section header
- **THEN** the tag list is hidden with a smooth animation and only the header remains visible

#### Scenario: Expand the tag section
- **WHEN** user clicks the expand toggle on a collapsed Tags section
- **THEN** the tag list slides open with a smooth animation

### Requirement: Clicking a tag filters the repository list
The system SHALL allow users to click on a tag in the sidebar to filter the repository list to show only repositories that have that tag assigned. Multiple tags MAY be selected simultaneously (OR logic: show repos that have ANY of the selected tags).

#### Scenario: Select a single tag
- **WHEN** user clicks on tag "work" in the sidebar
- **THEN** the repository list shows only repositories that have the "work" tag assigned, and the tag appears visually selected/highlighted in the sidebar

#### Scenario: Select multiple tags
- **WHEN** user clicks on tag "work" and then also clicks on tag "personal"
- **THEN** the repository list shows repositories that have either "work" OR "personal" tags

#### Scenario: Deselect a tag
- **WHEN** user clicks on an already-selected tag
- **THEN** the tag is deselected; the repo list updates to remove that filter

#### Scenario: Clear all tag filters
- **WHEN** user clicks a "Clear" or "All" action in the tag section
- **THEN** all tag selections are removed and the full repo list is shown

### Requirement: Inline tag creation from sidebar
The system SHALL provide a "+" button in the Tags section header that opens a compact inline form (or popover) for creating a new tag without leaving the current page.

#### Scenario: Create tag from sidebar
- **WHEN** user clicks "+" in the Tags section, enters "hackathon", picks a colour, and confirms
- **THEN** the tag is created and immediately appears in the sidebar tag list with count 0

### Requirement: Tag context menu for rename and delete
The system SHALL provide a right-click (or long-press / secondary action) context menu on each tag in the sidebar with options to Rename and Delete the tag.

#### Scenario: Rename via context menu
- **WHEN** user right-clicks a tag and selects "Rename"
- **THEN** the tag name becomes editable inline; on confirmation the tag is renamed

#### Scenario: Delete via context menu
- **WHEN** user right-clicks a tag and selects "Delete"
- **THEN** a confirmation prompt appears; on confirmation the tag and all its associations are deleted

### Requirement: Tag list updates in real-time
The sidebar tag list SHALL update reactively when tags are created, renamed, deleted, or when repo-tag assignments change. No manual refresh SHALL be required.

#### Scenario: Tag created elsewhere updates sidebar
- **WHEN** user creates a tag via the tag assignment popover on a repo card
- **THEN** the sidebar tag list immediately reflects the new tag

#### Scenario: Tag count updates on assignment
- **WHEN** user assigns tag "work" to a new repo
- **THEN** the count next to "work" in the sidebar increments by 1
