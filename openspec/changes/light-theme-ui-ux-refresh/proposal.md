## Why

Current Rust Harbor UI is visually dense and dark-heavy, which reduces readability and scanning speed for day-to-day repository management. We need a light-first redesign now to improve usability and accessibility, while establishing a tokenized theming foundation for future dark mode support.

## What Changes

- Redesign application visuals to a light-first interface across sidebar, toolbar, repository cards/list, dialogs, popovers, and preview panels.
- Replace low-contrast dark glass patterns with accessible light surfaces, clearer borders, and consistent text hierarchy.
- Standardize typography scale and font pairings for better readability in data-dense dashboard screens.
- Introduce a shared design token layer (colors, semantic text roles, spacing, radii, elevation, motion) to ensure UI consistency.
- Define component-level UX rules for states (hover, focus, active, disabled, loading, empty, error) and keyboard accessibility.
- Preserve current functional behavior while improving visual structure and interaction clarity.
- Prepare a non-breaking architecture path for future dark mode toggle via token/theme switching.

## Capabilities

### New Capabilities
- `light-theme-foundation`: Define and apply a light-first design token system and accessibility contrast baseline across the app.
- `dashboard-ui-ux-refresh`: Redesign core Rust Harbor dashboard UI components and interaction patterns for clarity, readability, and consistent UX.

### Modified Capabilities
- (none)

## Impact

- Affected frontend areas:
  - `src/app.css` (global tokens/theme variables)
  - `src/routes/+layout.svelte` (sidebar, tags area, dialogs, navigation chrome)
  - `src/routes/+page.svelte` (toolbar, cards/list, popovers, filters, preview panel)
  - `src/lib/components/custom/GroupHeader.svelte` and shared UI primitives where needed
- No backend command/API behavior changes expected.
- No database schema changes expected.
- New dependency usage may include typography/font assets and optional utility refinements for accessibility.
