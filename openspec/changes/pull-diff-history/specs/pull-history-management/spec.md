## ADDED Requirements

### Requirement: Delete individual pull history entry
The system SHALL allow users to delete a single pull history entry and all its associated file records.

#### Scenario: Delete a single entry
- **WHEN** the user clicks the delete button on a pull history card and confirms the action
- **THEN** the system SHALL delete the `pull_history` row and all associated `pull_history_files` rows, and the entry SHALL be removed from the list

#### Scenario: Delete without confirmation for single entry
- **WHEN** the user clicks delete on a pull history entry
- **THEN** the system SHALL show a confirmation prompt before deleting (destructive action)

### Requirement: Bulk delete selected pull history entries
The system SHALL allow users to select multiple pull history entries and delete them in a single operation.

#### Scenario: Selecting entries for bulk delete
- **WHEN** the user enables multi-select mode on the pull history page
- **THEN** each pull history card SHALL show a checkbox; the user can select one or more entries

#### Scenario: Bulk delete selected entries
- **WHEN** the user has selected one or more entries and clicks "Delete Selected"
- **THEN** the system SHALL show a confirmation prompt and, upon confirmation, delete all selected `pull_history` rows and their associated `pull_history_files` rows

### Requirement: Clear all pull history
The system SHALL provide a "Clear All History" action that removes every pull history entry from the database.

#### Scenario: Clear all history
- **WHEN** the user clicks "Clear All History" and confirms the action in the confirmation dialog
- **THEN** the system SHALL delete all rows from `pull_history` and `pull_history_files`, and the list SHALL show the empty state

#### Scenario: Clear all history with no entries
- **WHEN** there are no pull history entries and the user attempts to clear history
- **THEN** the "Clear All History" button SHALL be disabled

### Requirement: History entry count displayed
The system SHALL display the total number of pull history entries on the pull history page header.

#### Scenario: Count shown in header
- **WHEN** the pull history page is loaded
- **THEN** the page header SHALL show "N pulls recorded" where N is the total count of history entries (or filtered count when a filter is active)
