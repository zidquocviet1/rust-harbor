## ADDED Requirements

### Requirement: Groups render items incrementally up to a page size
The repository list SHALL initially render at most `PAGE_SIZE_LIST` (20) items in list view and `PAGE_SIZE_GRID` (9) items in grid view per group, rather than all items at once.

#### Scenario: Large group is capped on initial render
- **WHEN** a language group contains 50 repositories and the view is in list mode
- **THEN** only the first 20 repository cards are rendered in the DOM

#### Scenario: Small group renders fully
- **WHEN** a language group contains 8 repositories and the view is in list mode
- **THEN** all 8 repository cards are rendered (no artificial cap)

#### Scenario: Grid mode uses different page size
- **WHEN** a language group contains 30 repositories and the view is in grid mode
- **THEN** only the first 9 repository cards are rendered in the DOM

---

### Requirement: A scroll sentinel triggers incremental loading within a group
Each group that has more items than currently rendered SHALL display an invisible sentinel element after the last rendered card. When the sentinel enters the viewport, the next page of items SHALL be appended to that group.

#### Scenario: Sentinel triggers next page load
- **WHEN** the user scrolls down and the sentinel at the bottom of a group enters the viewport
- **THEN** the next page of items (up to PAGE_SIZE per group) is appended to that group's rendered list

#### Scenario: Sentinel disappears after full render
- **WHEN** all items in a group have been rendered
- **THEN** the sentinel element is removed from the DOM for that group

#### Scenario: Multiple groups load independently
- **WHEN** the user scrolls through a page with multiple groups
- **THEN** each group loads its own additional pages independently without affecting other groups

---

### Requirement: A "Show more" button provides keyboard-accessible fallback
Each group that has unrendered items SHALL render a visible "Show N more" button below the sentinel.

#### Scenario: Button shows remaining count
- **WHEN** a group has 35 items and 20 are rendered
- **THEN** a button reading "Show 15 more" is visible below the rendered items

#### Scenario: Button loads next page on click
- **WHEN** the user clicks the "Show N more" button
- **THEN** the next page of items is appended and the button count updates or disappears

---

### Requirement: Per-group visible counts reset when filters or view mode change
`groupVisibleCount` SHALL be reset to initial page sizes whenever the active filter set, search query, selected tags, or view mode changes.

#### Scenario: Search resets pagination
- **WHEN** the user types a new search query while a group has loaded 2 pages
- **THEN** that group's visible count resets to the initial page size for the current view mode

#### Scenario: View mode toggle resets pagination
- **WHEN** the user switches from list view to grid view
- **THEN** all groups reset to `PAGE_SIZE_GRID` (9) items

#### Scenario: Tag filter reset clears pagination
- **WHEN** the user removes a tag filter
- **THEN** all groups reset to initial page sizes
