## Why

The repository list page renders all items within visible groups at once, causing noticeable jank when opening the app with many repositories. Cards also appear abruptly with no motion cues, making filter/search transitions feel unpolished.

## What Changes

- Add per-group incremental loading: render an initial page of items (20 in list view, 9 in grid view), then auto-load more as the user scrolls to the bottom of each group
- Add staggered entrance animations for repo cards on initial load and after filter/search changes, using CSS custom properties (`--stagger-index`) and existing motion tokens
- Add smooth exit/crossfade transitions when the filtered set changes
- Extend motion CSS in `app.css` with reusable card-enter/card-exit keyframes that respect `prefers-reduced-motion`
- Reset per-group pagination state whenever filters, search query, tags, or view mode changes

## Capabilities

### New Capabilities

- `paginated-group-items`: Per-group incremental item rendering with scroll-triggered load-more, replacing the current flat render-all approach within each visible group
- `repo-card-animations`: Staggered entrance and smooth exit animations for repo cards, with a coherent motion system driven by CSS custom properties and Svelte transitions

### Modified Capabilities

<!-- None — no existing spec files found; group virtualization behavior is extended, not spec-level replaced -->

## Impact

- **`src/routes/+page.svelte`**: Core change — per-group `visibleCount` state map, scroll sentinel IntersectionObserver per group, stagger index injection, filter-change reset logic
- **`src/app.css`**: Add `@keyframes card-enter` / `card-exit`, motion utility classes, update motion tokens if needed
- No new npm dependencies required (native CSS + Svelte `transition:` directives)
- Existing group collapse (`transition:slide`), tag filters, Fuse.js search, grid/list toggle must remain fully functional
