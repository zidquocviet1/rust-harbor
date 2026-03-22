## ADDED Requirements

### Requirement: Capture pull event on successful git pull
The system SHALL capture a pull history record whenever a `git pull` operation completes successfully and introduces at least one new commit (i.e., the HEAD SHA changes).

#### Scenario: Pull introduces new commits
- **WHEN** a user triggers a git pull on a repository and the operation succeeds with new commits
- **THEN** the system SHALL record a `pull_history` row with: `repo_path`, `repo_name`, `branch`, `pulled_at` (UTC timestamp), `commit_before` (HEAD SHA before pull), `commit_after` (HEAD SHA after pull), and `files_changed_count`

#### Scenario: Pull is already up-to-date
- **WHEN** a git pull completes successfully but HEAD SHA is unchanged (already up to date)
- **THEN** the system SHALL NOT create a `pull_history` record

#### Scenario: Pull fails
- **WHEN** a git pull returns a non-zero exit code
- **THEN** the system SHALL NOT create a `pull_history` record

### Requirement: Capture per-file diff for each pull event
The system SHALL capture and persist per-file change metadata for every recorded pull history entry.

#### Scenario: Files changed during pull
- **WHEN** a pull history record is created and a file diff is computed between `commit_before` and `commit_after`
- **THEN** the system SHALL create one `pull_history_files` row per changed file containing: `file_path`, `change_type` (added/modified/deleted/renamed), `additions` (line count), `deletions` (line count), `diff_content` (unified diff string, max 500KB per file)

#### Scenario: Binary file changed during pull
- **WHEN** a changed file is detected as binary (numstat shows `-` for additions and deletions)
- **THEN** the system SHALL store a `pull_history_files` row with `additions = 0`, `deletions = 0`, and `diff_content = "[binary file]"`

#### Scenario: Diff capture fails
- **WHEN** the diff subprocess fails after a successful pull
- **THEN** the system SHALL still persist the `pull_history` row and log a warning, but `files_changed_count = 0` and no `pull_history_files` rows are created

### Requirement: Persist history independent of repository watchlist
The system SHALL retain pull history records permanently, regardless of whether the source repository remains in the user's watched folders.

#### Scenario: Repository removed from watchlist
- **WHEN** a user removes a repository folder from the watched folders
- **THEN** all `pull_history` and `pull_history_files` records for that repository SHALL remain intact in the database

#### Scenario: History survives app restart
- **WHEN** the application is restarted
- **THEN** all previously captured pull history records SHALL be retrievable from the database

### Requirement: Pull result returns history reference
The system SHALL return a structured result from the `git_pull` Tauri command that includes whether a history record was created.

#### Scenario: Pull with new commits
- **WHEN** `git_pull` succeeds and creates a history record
- **THEN** the command SHALL return `{ output: String, history_id: Some(i64) }`

#### Scenario: Pull already up to date
- **WHEN** `git_pull` succeeds but no new commits were introduced
- **THEN** the command SHALL return `{ output: String, history_id: None }`
