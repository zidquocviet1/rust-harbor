## ADDED Requirements

### Requirement: Git Binary Path Configuration
The system SHALL allow users to specify a custom path to the Git executable if they want to override the system default.

#### Scenario: Update Git path
- **WHEN** the user provides a path to "/usr/local/bin/git" in settings
- **THEN** all subsequent Git operations use that specific binary

### Requirement: Git Version Verification
The system SHALL provide a "Verify" button that checks if the configured Git binary is accessible and returns its version.

#### Scenario: Successful verification
- **WHEN** the user clicks "Verify Git" and the binary is valid
- **THEN** the system displays the Git version (e.g., "Git 2.40.1 detected")
