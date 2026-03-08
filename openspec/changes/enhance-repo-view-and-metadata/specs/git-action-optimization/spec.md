## ADDED Requirements

### Requirement: Direct Fetch/Pull Actions
In the list view, the dashboard must prioritize direct Git actions ('Fetch' and 'Pull') over repository-level actions ('Open Folder' and 'Refresh').

#### Scenario: Branch-specific Sync
- **WHEN** The 'Pull' button in a repository's list-view row is clicked.
- **THEN** The system must execute `git pull` only for the currently active branch, ensuring no external merge conflicts across separate branches are accidentally triggered.

### Requirement: Lightweight Remote Connectivity Probe
The backend must verify remote connectivity by a short-lived network probe (e.g., `git ls-remote`) instead of just checking for an 'origin' URL.

#### Scenario: Inaccessible Remote Detection
- **WHEN** A repository has a valid origin URL but is deleted from the cloud or the user is disconnected.
- **THEN** The remote connectivity status must update to "Offline" or "Unauthorized" after a 5-second timeout, rather than remaining as "Connected".
