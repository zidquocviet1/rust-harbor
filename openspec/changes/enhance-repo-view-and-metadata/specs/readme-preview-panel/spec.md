## ADDED Requirements

### Requirement: Integrated README Inspector
The dashboard must provide a non-modal sliding panel on the right that renders the repository's README file content.

#### Scenario: Visual Inspection
- **WHEN** A repository is clicked.
- **THEN** A side panel slides out from the right containing the README file, rendered as Markdown by default.

### Requirement: Multi-view Content Rendering
The README preview panel must allow users to toggle between Markdown, Plain Text, and Unified (Header summary) views.

#### Scenario: Metadata Review
- **WHEN** The "Unified" toggle is selected in the preview panel.
- **THEN** The panel shows a high-level summary of the repository's headers, description, and key metadata instead of just the rendered Markdown.
