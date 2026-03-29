## MODIFIED Requirements

### Requirement: Pull History detail view shows commit and file changes
The system SHALL allow users to expand a pull history entry to view detailed commit information, a list of all changed files, and an AI-generated summary of the changes.

#### Scenario: Expanding a pull history entry
- **WHEN** the user clicks on a pull history card
- **THEN** the card SHALL expand inline (or navigate to a detail panel) to show: full before/after commit SHAs, commit message for the after commit, author name and date, a list of all changed files grouped by change type, and an AI summary panel

#### Scenario: File list in detail view
- **WHEN** a pull history entry is expanded
- **THEN** each changed file SHALL be shown with: file path, change type badge (Added/Modified/Deleted/Renamed in distinct colors), additions count (+N in green), and deletions count (-N in red)

#### Scenario: AI summary panel — not yet generated
- **WHEN** a pull history entry is expanded and no AI summary exists for it
- **THEN** the summary panel SHALL show a "Summarize with AI" button; if AI settings are not configured, the button SHALL be disabled with a tooltip "Configure an AI provider in Settings first"

#### Scenario: AI summary panel — generating
- **WHEN** the user clicks "Summarize with AI" and the request is in progress
- **THEN** the button SHALL be replaced with a loading spinner and the label "Generating summary…"; the button SHALL be non-interactive during generation

#### Scenario: AI summary panel — summary available
- **WHEN** a pull history entry is expanded and an AI summary exists (cached or just generated)
- **THEN** the summary panel SHALL render the summary text as formatted markdown and show a "Regenerate" button to overwrite the cached result

#### Scenario: AI summary generation error
- **WHEN** the AI API call fails during summary generation
- **THEN** the summary panel SHALL display the error message returned by the backend and show the "Summarize with AI" button again so the user can retry

#### Scenario: Regenerating a summary
- **WHEN** the user clicks "Regenerate" on an existing summary
- **THEN** the system SHALL call `generate_pull_summary` ignoring the cache, overwrite the stored summary, and display the new result
