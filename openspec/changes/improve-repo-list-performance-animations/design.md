## Context

The repository list page (`src/routes/+page.svelte`) is a ~90KB monolithic Svelte 5 component. Repositories are grouped by language, and a group-level IntersectionObserver already virtualizes entire groups (collapsing off-screen groups to placeholder divs). However, within each visible group all items are rendered fully in the DOM at once — this is the source of jank with large repo counts.

Animations currently exist only as ad-hoc Tailwind classes (`hover:-translate-y-0.5`, `transition:slide` for group collapse). There is no systematic approach to entrance/exit motion.

Design tokens in `app.css`: `--motion-fast: 140ms`, `--motion-standard: 220ms`. The app respects `prefers-reduced-motion`. No new npm packages should be added.

## Goals / Non-Goals

**Goals:**
- Reduce initial DOM node count for large repo lists by rendering items in pages within each group
- Auto-load more items when the user scrolls near the bottom of a group (scroll sentinel pattern)
- Add staggered card entrance animations on first render and after filter/search changes
- Add smooth exit crossfade when filter set changes
- Keep a unified CSS motion system in `app.css` instead of scattered inline styles

**Non-Goals:**
- True virtual/windowed scrolling (recycled DOM nodes) — overkill for typical repo counts (<500), adds significant complexity
- Server-side or backend pagination — all repo data is already in the frontend store
- Applying animations to sidebar, settings, or other routes in this change — repo list page only
- Skeleton loading states (separate concern)

## Decisions

### 1. Incremental loading: per-group page size + scroll sentinel

**Decision**: Use a `Map<groupKey, number>` called `groupVisibleCount` to track how many items each group renders. Initially render `PAGE_SIZE_LIST = 20` (list) or `PAGE_SIZE_GRID = 9` (grid) items. Place an invisible sentinel `<div>` at the end of each group's rendered items; when it enters the viewport via IntersectionObserver, increment that group's count by one page.

**Why over alternatives:**
- *True virtualization (svelte-virtual-list)*: Requires measuring item heights, breaks with variable-height cards, conflicts with existing group collapse logic. Too complex.
- *Single global "load more" button*: Doesn't work well with grouped layout — a group at the bottom of the page would never auto-expand.
- *Render all + CSS contain*: `content-visibility: auto` with `contain-intrinsic-size` is promising but browser support and interaction with Svelte transitions is unpredictable.

**Reset trigger**: `groupVisibleCount` resets to initial page sizes whenever `filteredRepos`, `viewMode`, or any filter input changes (via `$effect` watching those values).

### 2. Staggered entrance: CSS custom property + keyframe

**Decision**: Add a `@keyframes card-enter` in `app.css` that translates from `0 8px` to `0 0` and fades from `0` to `1` over `var(--motion-standard)`. Apply via a `.repo-card-enter` class. Inject `--stagger-index` as an inline style on each card; the animation-delay is `calc(var(--stagger-index, 0) * 30ms)`. Cap stagger at 12 items (index ≥ 12 uses 360ms delay) to avoid long waits.

**Why over Svelte `transition:` directive:**
- Svelte `in:fly` / `in:fade` requires wrapping each item in `{#key}` blocks or using `animate:flip` — these fight with the existing `{#each}` group rendering structure.
- CSS keyframes are trivially `prefers-reduced-motion` compatible: wrap the entire keyframe in `@media (prefers-reduced-motion: no-preference)`.
- Pure CSS means no Svelte re-render overhead during animation.

### 3. Filter change exit: brief opacity fade on container

**Decision**: When filters/search changes, apply a one-frame CSS class `filtering` to the grid/list container that sets `opacity: 0.4` and `pointer-events: none` for `var(--motion-fast)` (140ms), then removes it. New items enter with card-enter animation. This avoids complex FLIP animations while giving a clear visual cue that content is changing.

**Why not FLIP/animate:flip**: The grouped structure with variable item counts makes FLIP unreliable. The brief dim-and-repopulate approach is simpler and sufficient.

### 4. Sentinel observer: reuse existing IntersectionObserver infrastructure

**Decision**: Create a second IntersectionObserver instance (`loadMoreObserver`) dedicated to scroll sentinels. Each group renders a sentinel `<div data-group-key={key}>` as its last child when `visibleCount < totalCount`. The observer callback reads `dataset.groupKey` and increments `groupVisibleCount`.

The existing `groupVisibilityObserver` (800px margin, group-level) remains untouched.

## Risks / Trade-offs

- **Stagger cap at 12**: Users with small screens may see only 3-4 items but they'll all animate. Users with large screens and 20+ items will see only the first 12 stagger; remaining items animate simultaneously. This is acceptable.
- **Filter flash**: The 140ms opacity dip may feel abrupt on very fast machines. Mitigation: keep the duration at `--motion-fast` (140ms) which is imperceptible on modern hardware.
- **`groupVisibleCount` reset on view mode change**: Switching grid↔list resets all page counts to initial values. This is correct behavior (page sizes differ) but the user loses their scroll position context within a group. Mitigation: only reset when view mode actually changes, not on every render.
- **Multiple IntersectionObserver instances**: Two observers (group visibility + load-more sentinels) is fine; browsers handle dozens of observers efficiently. Sentinels are unobserved once a group is fully loaded.

## Migration Plan

1. Add CSS keyframes and utility classes to `app.css` (no functional impact)
2. Add `groupVisibleCount` state and `loadMoreObserver` to `+page.svelte`
3. Modify the group render loop to slice items by `groupVisibleCount[key]` and append sentinel
4. Inject `--stagger-index` on each card's style attribute
5. Add filter-change `$effect` to reset counts and trigger fade class
6. Manual test: large repo list, small repo list, filter changes, view mode toggle, group collapse
7. Remove `.filtering` class and stagger cap only if UX review calls for it

**Rollback**: All changes are frontend-only and contained to two files. Reverting is a simple `git revert`.

## Open Questions

- Should "load more" also support an explicit button fallback for accessibility (keyboard users who can't scroll to trigger the sentinel)? Recommend yes — add a visually-styled "Show N more" button below the sentinel as fallback.
- Should the stagger delay be configurable via a CSS token (e.g., `--stagger-step: 30ms`) in `app.css`? Recommend yes for future consistency.
