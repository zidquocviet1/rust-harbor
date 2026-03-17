## ADDED Requirements

### Requirement: Get Supported Editors
The system SHALL provide a mechanism to retrieve a list of supported editors and IDEs that are currently installed on the user's machine (supporting macOS, Windows, and Linux).

#### Scenario: Successfully retrieve installed editors
- **WHEN** the frontend requests the list of installed editors
- **THEN** the system returns a list of installed editors, including their display name, internal identifier, and icon reference

### Requirement: Open Repository in Editor
The system SHALL allow the user to open the root directory of a selected repository in a specific, installed editor across all supported platforms (macOS, Windows, and Linux).

#### Scenario: Open repository in VS Code
- **WHEN** the user selects "Open in VS Code" for a repository
- **THEN** the system executes the command to open the repository path in Visual Studio Code

#### Scenario: Open repository in JetBrains IDE
- **WHEN** the user selects "Open in IntelliJ IDEA" for a repository
- **THEN** the system executes the command to open the repository path in IntelliJ IDEA

### Requirement: Dynamic Display of Editor Options
The user interface SHALL only display "Open in" options for editors that have been detected as installed on the system.

#### Scenario: Editor not installed
- **WHEN** an editor (e.g., Zed) is not installed on the system
- **THEN** the option to "Open in Zed" SHALL NOT be visible in the UI
