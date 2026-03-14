## ADDED Requirements

### Requirement: Display Repository Count per Folder
The system SHALL display the total number of Git repositories discovered within each watched folder.

#### Scenario: View repo count
- **WHEN** the user opens the Workspace Settings page
- **THEN** each watched folder entry shows a badge with the number of repositories found (e.g., "12 Repos")

### Requirement: Last Scan Status Indicator
The system SHALL display the timestamp and status (success/failure) of the last scan for each watched directory.

#### Scenario: Successful scan indicator
- **WHEN** a folder has been successfully scanned
- **THEN** the entry shows a green "Synced" status with a relative timestamp (e.g., "5m ago")

#### Scenario: Scanned with errors indicator
- **WHEN** a folder scan encountered accessibility or permission issues
- **THEN** the entry shows a red "Warning" status with details on the failure
