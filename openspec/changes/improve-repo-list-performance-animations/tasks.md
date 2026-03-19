## 1. CSS Motion System (app.css)

- [x] 1.1 Add `--stagger-step: 30ms` custom property to `:root` in `app.css`
- [x] 1.2 Add `@keyframes card-enter` (fade from 0→1 + translate from 8px→0) wrapped in `@media (prefers-reduced-motion: no-preference)`
- [x] 1.3 Add `.repo-card-enter` utility class that applies `card-enter` animation using `var(--motion-standard)` with `ease-out` timing
- [x] 1.4 Add `.filtering` utility class that sets `opacity: 0.4` and `pointer-events: none` with `var(--motion-fast)` transition (respects reduced-motion by omitting animation in reduced-motion media query)

## 2. Incremental Loading State (page.svelte)

- [x] 2.1 Define `PAGE_SIZE_LIST = 20` and `PAGE_SIZE_GRID = 9` constants near the existing height constants
- [x] 2.2 Add `groupVisibleCount = new Map<string, number>()` reactive state variable
- [x] 2.3 Write a helper `getPageSize(): number` that returns `PAGE_SIZE_LIST` or `PAGE_SIZE_GRID` based on current `viewMode`
- [x] 2.4 Write a helper `initGroupCounts(groups)` that resets `groupVisibleCount` to `getPageSize()` for every group key
- [x] 2.5 Add `$effect` that watches `filteredRepos`, `viewMode`, `selectedTags`, and `searchQuery` and calls `initGroupCounts` to reset all group counts

## 3. Scroll Sentinel Observer (page.svelte)

- [x] 3.1 Create a `loadMoreObserver` IntersectionObserver instance (threshold 0.1, no root margin) that reads `dataset.groupKey` from the sentinel element and increments `groupVisibleCount` for that group by `getPageSize()`, capped at the group's total item count
- [x] 3.2 Add `onDestroy` cleanup to disconnect `loadMoreObserver`

## 4. Group Render Loop — Slicing & Sentinel (page.svelte)

- [x] 4.1 In the group `{#each}` render block, slice the group's repo array to `groupVisibleCount.get(groupKey) ?? getPageSize()` items before rendering cards
- [x] 4.2 After the sliced card list, add the invisible sentinel `<div>` with `data-group-key={groupKey}` when `visibleCount < group.repos.length`; use Svelte `use:` action or `$effect` to observe/unobserve it with `loadMoreObserver`
- [x] 4.3 After the sentinel, add the "Show N more" button that displays remaining count and calls `incrementGroupCount(groupKey)` on click; hide the button when group is fully rendered
- [x] 4.4 Style the "Show N more" button using existing design tokens (ghost/secondary variant, centered, consistent with the app's button style)

## 5. Staggered Card Animations (page.svelte)

- [x] 5.1 In the card render loop, add `class="repo-card-enter"` to each repo card's root element
- [x] 5.2 Inject `style="--stagger-index: {Math.min(cardIndex, 12)}"` on each card's root element, where `cardIndex` is the card's position within the current rendered batch
- [x] 5.3 For "load more" batches, ensure stagger indices start from 0 for newly appended cards (track batch offset separately from global index)

## 6. Filter Change Fade (page.svelte)

- [x] 6.1 Add a boolean reactive variable `isFiltering = false`
- [x] 6.2 In the filter-change `$effect` (from task 2.5), set `isFiltering = true`, then use `setTimeout(fn, var(--motion-fast) equivalent = 140)` to set it back to `false` after the fade duration
- [x] 6.3 Bind `isFiltering` to the `filtering` CSS class on the repo list container element

## 7. Verification & Cleanup

- [x] 7.1 Test with a large repo count (>50): verify initial render is fast and items load incrementally on scroll
- [x] 7.2 Test filter/search/tag changes: verify container dims briefly and cards re-enter with stagger animation
- [x] 7.3 Test group collapse/expand: verify pagination state is preserved for collapsed groups and collapse animation is unaffected
- [x] 7.4 Test view mode toggle (grid↔list): verify pagination resets and correct page sizes apply
- [x] 7.5 Test with `prefers-reduced-motion: reduce` in OS settings: verify cards appear instantly and container does not dim
- [x] 7.6 Test "Show N more" button with keyboard navigation (Tab + Enter)
- [x] 7.7 Remove any debug `console.log` statements; verify no Svelte warnings in the browser console
