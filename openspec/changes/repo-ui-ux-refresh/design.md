## Context

The app has multiple UI touchpoints involved in navigation and repository detail presentation. Users experience visual inconsistency in the sidebar, redundant folder path display, and a raw view that is no longer needed. The unified view needs richer metadata (tags and linkable remotes). Repository detail loading currently causes UI stutter when heavy resources (images and metadata) load in bulk. The stack is SvelteKit with Tailwind utilities and a Tauri backend.

## Goals / Non-Goals

**Goals:**
- Centralize and update application name/icon usage across the UI.
- Simplify folder navigation by removing redundant path display.
- Remove raw view and enrich unified view with tags and hyperlink remotes.
- Reduce UI stutter by progressively loading heavy resources in repository detail.

**Non-Goals:**
- No backend data model changes beyond existing tag/remote data.
- No new external services or analytics.
- No redesign of non-related pages beyond the specified areas.

## Decisions

- **Branding source of truth**: Define app name and icon references in a single UI config module so the sidebar/header and any window title usage pull from one place. This avoids drift and makes future updates safer.
  - Alternatives considered: Hard-coded strings per component (rejected: inconsistent and brittle).

- **Folder navigation simplification**: Remove detailed path text from the folder list UI when a folder icon is present and the folder name is already shown. Keep full path accessible only via existing interactions (e.g., open folder or tooltip if already present).
  - Alternatives considered: Truncate paths (rejected: still noisy and not aligned with request).

- **View modes**: Remove raw view entry points and ensure unified view renders tag chips and hyperlink remote URLs. Use consistent link styles and target behavior aligned with existing app patterns.
  - Alternatives considered: Keep raw view behind a toggle (rejected: explicit request to remove).

- **Performance strategy for repository detail**: Switch to progressive rendering and lazy-loading for heavy assets.
  - Use IntersectionObserver (or equivalent Svelte action) to defer image loading until visible.
  - Decode images asynchronously and show lightweight placeholders to prevent layout thrash.
  - Defer non-critical resource fetches to idle time or after initial content paint.
  - Alternatives considered: Preload everything (rejected: current stutter issue).

## Risks / Trade-offs

- **[Risk]** Lazy-loading may delay visibility of some images → **Mitigation**: Use clear placeholders and prioritize above-the-fold assets.
- **[Risk]** Removing raw view could affect power users → **Mitigation**: Ensure unified view exposes required metadata (tags, remotes) and validate with users.
- **[Trade-off]** Slightly more complex UI logic for progressive loading → **Mitigation**: Keep helpers isolated and documented.
