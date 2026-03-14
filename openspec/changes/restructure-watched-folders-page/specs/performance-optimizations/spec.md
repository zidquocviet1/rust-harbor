## ADDED Requirements

### Requirement: Configure Exclusion Patterns
The system SHALL allow users to define project-wide or per-workspace exclusion patterns (globs) to skip during scanning.

#### Scenario: Add exclusion pattern
- **WHEN** the user adds "node_modules/**" to the Performance Settings
- **THEN** the scanning engine skips any directory matching that pattern

### Requirement: Recursive Scan Depth Limit
The system SHALL provide an option to limit the recursion depth for folder scanning to prevent traversing extremely deep directory structures.

#### Scenario: Set max depth
- **WHEN** the user sets a global recursion limit of 3
- **THEN** folders deeper than 3 levels from the root are ignored regardless of content

### Requirement: Auto-Refresh Toggle and Interval
The system SHALL allow users to enable/disable background auto-refresh and customize the polling interval.

#### Scenario: Disable auto-refresh
- **WHEN** the user toggles auto-refresh to "Off"
- **THEN** the background scanner stops periodic polling until manually triggered or app restart
