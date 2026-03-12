## ADDED Requirements

### Requirement: Progressive loading for heavy resources
Repository detail views SHALL load heavy resources (such as images) progressively so initial UI content can render without waiting for those resources.

#### Scenario: Initial detail renders before heavy resources
- **WHEN** a repository detail view is opened
- **THEN** the main layout renders before heavy resources finish loading

### Requirement: Lazy-load offscreen images
Images in repository detail SHALL NOT load until they are near the viewport.

#### Scenario: Offscreen images defer loading
- **WHEN** an image is below the fold in repository detail
- **THEN** it is loaded only after it approaches the viewport
