## MODIFIED Requirements

### Requirement: git pull command captures before and after state
The `git_pull` Tauri command SHALL capture the repository HEAD SHA immediately before and immediately after executing the pull, compute the diff between those two SHAs if they differ, persist the result to the pull history database tables, and return a structured result containing the git output string and the created history entry ID (if any).

#### Scenario: Successful pull with new commits
- **WHEN** the `git_pull` command is invoked for a repository path
- **THEN** the system SHALL:
  1. Read the current HEAD SHA via `git rev-parse HEAD`
  2. Execute `git pull origin HEAD`
  3. Read the new HEAD SHA via `git rev-parse HEAD`
  4. If the SHAs differ, compute the diff via `git diff <before>..<after> --numstat` and `git diff <before>..<after> --unified=3`
  5. Persist the pull history record and file records in a single SQLite transaction
  6. Return `PullResult { output: String, history_id: Some(i64) }`

#### Scenario: Successful pull already up to date
- **WHEN** the `git_pull` command is invoked and the pull reports "Already up to date"
- **THEN** the system SHALL return `PullResult { output: String, history_id: None }` without creating any database records

#### Scenario: Pull command fails
- **WHEN** `git pull origin HEAD` exits with a non-zero code
- **THEN** the system SHALL return an error result without creating any database records, preserving existing behavior
