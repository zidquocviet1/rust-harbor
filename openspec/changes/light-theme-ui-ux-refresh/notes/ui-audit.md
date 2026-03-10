## UI Audit (Light Theme Refresh)

### Scope Audited
- `src/app.css`
- `src/routes/+layout.svelte`
- `src/routes/+page.svelte`
- `src/lib/components/custom/GroupHeader.svelte`

### 1) Current Color / Font / Spacing Inventory

- Color usage patterns found before refactor:
  - Dark-heavy primitives: `bg-black/*`, `border-white/*`, `bg-white/5`, `prose-invert`
  - Brand emphasis via primary indigo accents
  - Frequent translucent glass classes without semantic token mapping
- Typography patterns found before refactor:
  - `DM Sans` global, heavy display styling in headers
  - Mixed small metadata sizes with low-contrast muted text
- Spacing / shape patterns:
  - Rounded 2xl surfaces and compact control clusters
  - Multiple one-off spacing/opacity combinations

### 2) Dark-only Styles Mapped to Semantic Targets

- `--background`, `--foreground`, `--card`, `--popover`, `--border`, `--muted*` mapped to light-first values.
- Added semantic role tokens:
  - `--surface-1`, `--surface-2`, `--surface-3`, `--surface-overlay`
  - `--text-primary`, `--text-secondary`, `--text-muted`
  - spacing/motion tokens for consistency
- Replaced key dark patterns in layout/page overlays and controls with light surfaces and slate borders.

### 3) Contrast Hotspots (Before) and Fix Direction

- Hotspots observed:
  - Metadata text with `/40` and `/60` opacity on low-contrast surfaces
  - Overlay/dialog text against dark translucent layers
  - Tag/filter chips using very low-opacity fills (`bg-white/5`)
- Fix direction applied:
  - Increase base surface opacity to `bg-white/80+` where required
  - Raise muted text contrast to readable levels
  - Use explicit borders (`border-slate-200/70+`) for control separation

### 4) Accessibility Notes

- `prefers-reduced-motion` rule added globally.
- Context menu dismiss backdrop in layout switched to semantic `<button>` to reduce non-semantic click target issues.
- Remaining known baseline TS/a11y warnings in project still exist in files not introduced by this change and should be handled in a follow-up quality pass.
