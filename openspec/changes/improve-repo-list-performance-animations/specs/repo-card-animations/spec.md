## ADDED Requirements

### Requirement: Repo cards animate in with a staggered entrance
Each repo card SHALL animate in using a fade + upward translate keyframe animation when it first appears in the DOM. Cards within a batch SHALL stagger their animation start using a CSS custom property `--stagger-index`.

#### Scenario: First batch animates on page load
- **WHEN** the repository list page loads and initial cards are rendered
- **THEN** each card fades in and translates from 8px below to its final position, with each card delayed by `calc(var(--stagger-index, 0) * var(--stagger-step, 30ms))`

#### Scenario: Stagger index is capped to avoid long waits
- **WHEN** a group renders more than 12 items in one batch
- **THEN** cards at index ≥ 12 all use a delay of `calc(12 * var(--stagger-step, 30ms))` (360ms cap)

#### Scenario: Load-more batch animates with fresh stagger
- **WHEN** a new page of items is appended via scroll sentinel or button click
- **THEN** the newly appended cards animate in with stagger indices starting from 0

---

### Requirement: Entrance animations are disabled when prefers-reduced-motion is set
The card entrance keyframe animation SHALL be wrapped in `@media (prefers-reduced-motion: no-preference)`. When the user has enabled reduced motion, cards SHALL appear instantly without animation.

#### Scenario: Reduced motion disables entrance animation
- **WHEN** the operating system has `prefers-reduced-motion: reduce` set
- **THEN** repo cards appear immediately at full opacity with no transform

#### Scenario: Normal motion enables entrance animation
- **WHEN** `prefers-reduced-motion` is not set or is `no-preference`
- **THEN** repo cards animate in with the staggered fade+translate effect

---

### Requirement: Filter and search changes trigger a brief container fade
When the active filter set, search query, or selected tags change, the repo list container SHALL briefly dim (opacity 0.4) for `var(--motion-fast, 140ms)` before the new cards render, providing a visual cue that content is refreshing.

#### Scenario: Search input triggers container fade
- **WHEN** the user changes the search query
- **THEN** the list container transitions to `opacity: 0.4` for 140ms, then returns to full opacity as new cards enter with their entrance animations

#### Scenario: Tag filter change triggers container fade
- **WHEN** the user selects or deselects a tag filter
- **THEN** the same brief dim-and-repopulate sequence occurs

#### Scenario: Reduced motion skips the fade
- **WHEN** `prefers-reduced-motion: reduce` is set and the user changes a filter
- **THEN** the container does NOT dim; new content appears immediately

---

### Requirement: A unified card motion CSS system is defined in app.css
The `app.css` file SHALL define reusable keyframe animations and CSS custom properties for the card motion system, rather than relying on scattered inline styles.

#### Scenario: card-enter keyframe is defined
- **WHEN** the `.repo-card-enter` class is applied to an element
- **THEN** the element plays the `card-enter` keyframe animation using `var(--motion-standard)` duration with `ease-out` timing

#### Scenario: Stagger step token is defined
- **WHEN** reviewing the CSS custom properties in `:root`
- **THEN** `--stagger-step: 30ms` is present alongside `--motion-fast` and `--motion-standard`
