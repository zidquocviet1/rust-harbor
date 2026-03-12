## Why

The current navigation and repository detail views feel inconsistent and heavy to use, with redundant information and performance hiccups when loading rich content. This change improves usability and visual coherence while reducing UI stutter during repository detail loads.

## What Changes

- Update application branding (icon and name) for clearer identity.
- Simplify folder navigation by removing detailed path display when a folder icon already provides access.
- Remove the raw view when opening a folder.
- Enhance unified view by showing tags and making remote links clickable.
- Improve repository detail loading to reduce UI stutter when heavy resources (e.g., images) load.

## Capabilities

### New Capabilities
- `app-branding`: Define the application name and icon update behavior across the UI.
- `sidebar-navigation-simplification`: Remove detailed folder path display in the sidebar navigation when redundant.
- `repository-view-modes`: Specify removal of raw view and updated unified view content (tags, hyperlinking).
- `repository-detail-performance`: Define performance expectations and loading behavior for heavy resources in repo detail.

### Modified Capabilities
- (none)

## Impact

- UI components: sidebar, folder navigation, unified view, repository detail view.
- Assets: app icon and name display strings.
- Potential data rendering logic for tags and remote links.
- Performance-sensitive loading paths for images/resources.
