## ADDED Requirements

### Requirement: Raw view is removed from folder opening
The UI SHALL NOT present a raw view option when a folder is opened.

#### Scenario: Folder open does not expose raw view
- **WHEN** the user opens a folder
- **THEN** no raw view entry or toggle is available

### Requirement: Unified view includes tags and hyperlink remotes
The unified view SHALL display repository tags and render remote URLs as clickable hyperlinks.

#### Scenario: Unified view shows tags and linkable remotes
- **WHEN** a repository is shown in unified view
- **THEN** its tags are visible and each remote URL is rendered as a hyperlink
