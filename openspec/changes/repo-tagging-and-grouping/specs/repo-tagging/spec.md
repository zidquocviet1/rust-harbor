## ADDED Requirements

### Requirement: User can create a custom tag
The system SHALL allow users to create a new tag by providing a name and selecting a colour from a curated palette. Tag names MUST be unique (case-insensitive). Tag names MUST be trimmed of leading/trailing whitespace and SHALL have a maximum length of 32 characters.

#### Scenario: Successful tag creation
- **WHEN** user provides a valid, unique tag name and selects a colour
- **THEN** the system persists the tag to the SQLite database and the tag appears in the sidebar tag list immediately

#### Scenario: Duplicate tag name
- **WHEN** user tries to create a tag with a name that already exists (case-insensitive match)
- **THEN** the system SHALL reject the creation and display an error toast "Tag already exists"

#### Scenario: Empty or whitespace-only tag name
- **WHEN** user submits a tag with an empty or whitespace-only name
- **THEN** the system SHALL reject the creation and display a validation error

### Requirement: User can assign tags to a repository
The system SHALL allow users to assign one or more existing tags to a repository. A repository MAY have zero or more tags. The assignment MUST be persisted in the `repo_tags` table using the repository's absolute path as the key.

#### Scenario: Assign a tag to a repo
- **WHEN** user selects a tag from the tag popover on a repository card
- **THEN** the tag-repo association is persisted and the tag badge appears on the repo card immediately

#### Scenario: Assign multiple tags
- **WHEN** user assigns multiple different tags to the same repository
- **THEN** all tags are persisted and all tag badges are visible on the repo card

#### Scenario: Assign a tag already assigned
- **WHEN** user attempts to assign a tag that is already assigned to the repo
- **THEN** the system SHALL treat this as a no-op (idempotent, no error)

### Requirement: User can remove a tag from a repository
The system SHALL allow users to remove a specific tag from a repository without deleting the tag itself.

#### Scenario: Remove a tag from a repo
- **WHEN** user clicks the remove/unassign action on a tag badge of a repo card
- **THEN** the association is removed from the database and the tag badge disappears from the repo card

### Requirement: User can delete a tag entirely
The system SHALL allow users to delete a tag. Deleting a tag MUST cascade-remove all associations in `repo_tags` for that tag.

#### Scenario: Delete a tag
- **WHEN** user triggers the delete action on a tag (via context menu or tag manager)
- **THEN** the tag is removed from the `tags` table, all `repo_tags` entries referencing it are deleted, and the tag disappears from the sidebar and all repo cards

### Requirement: User can rename a tag
The system SHALL allow users to rename an existing tag. The new name MUST follow the same uniqueness and validation rules as creation.

#### Scenario: Rename a tag successfully
- **WHEN** user provides a new valid unique name for an existing tag
- **THEN** the tag name is updated in the database and reflected in the sidebar and all repo cards

#### Scenario: Rename to an existing name
- **WHEN** user attempts to rename a tag to a name that already exists
- **THEN** the system SHALL reject the rename and show an error

### Requirement: Orphaned tag associations are cleaned up on refresh
The system SHALL automatically remove `repo_tags` entries whose `repo_path` no longer corresponds to a repository in the current cache after a watched-folder scan completes. Tags themselves (the `tags` table) SHALL NOT be auto-deleted.

#### Scenario: Repo removed from watched folders
- **WHEN** a watched folder is removed or a previously scanned repo directory is deleted, and a refresh/scan is triggered
- **THEN** all `repo_tags` entries for paths that are no longer in the scanned repo set are deleted; the tags remain available

#### Scenario: No orphans exist
- **WHEN** a scan completes and all `repo_tags` entries match existing repos
- **THEN** no deletions occur

### Requirement: Tags are included in RepoMetadata
The system SHALL populate a `tags` field (list of tag names) on each `RepoMetadata` object during the scan/cache-build process by querying the database.

#### Scenario: Repo with tags
- **WHEN** a repo has two tags assigned ("work", "rust-projects")
- **THEN** the `RepoMetadata.tags` field contains `["work", "rust-projects"]`

#### Scenario: Repo with no tags
- **WHEN** a repo has no tags assigned
- **THEN** the `RepoMetadata.tags` field is an empty list `[]`
