## 1. UI/UX Baseline Audit

- [x] 1.1 Inventory current color/font/spacing usage in `src/app.css`, `src/routes/+layout.svelte`, `src/routes/+page.svelte`, and shared custom components
- [x] 1.2 Identify hardcoded dark-only styles and map them to semantic token targets
- [x] 1.3 Document current contrast hotspots (metadata text, badges, borders, overlay text) that fail readability goals

## 2. Light Theme Token Foundation

- [x] 2.1 Define semantic design tokens in global styles for background, surfaces, borders, text roles, accent, and status roles
- [x] 2.2 Introduce semantic elevation and overlay tokens for cards, popovers, dialogs, and side panels
- [x] 2.3 Normalize radius, spacing, and motion timing tokens for consistent component behavior
- [x] 2.4 Replace hardcoded route-level color literals with semantic tokens in touched files

## 3. Typography and Readability System

- [x] 3.1 Define and apply consistent typography scale for title, section heading, body, metadata, and micro-label text
- [x] 3.2 Apply readable light-mode text contrast hierarchy (primary/secondary/muted/meta) across dashboard surfaces
- [x] 3.3 Standardize monospace usage for paths/code-like metadata only, with readable sizing and contrast
- [x] 3.4 Tune line-height and spacing on dense content blocks to improve scanability

## 4. Dashboard Shell and Navigation Refresh

- [x] 4.1 Refactor sidebar shell in `+layout.svelte` to light-first surfaces, borders, and selected states
- [x] 4.2 Refresh top toolbar controls in `+page.svelte` (view toggle, group-by, refresh, search/filter controls) for consistency
- [x] 4.3 Update dialog, context menu, and popover surfaces to the same light-mode visual language
- [x] 4.4 Ensure overlay layering/backdrop treatments remain clear without dark-heavy blur dependence

## 5. Repository Views and Tag UX Refresh

- [x] 5.1 Refresh repository card (grid) visuals for clearer information priority and interaction affordance
- [x] 5.2 Refresh repository row (list) visuals for dense but readable scanning
- [x] 5.3 Align tag badges, dots, and counts with refreshed tokenized color roles
- [x] 5.4 Refine tag assignment popover readability and selected-state clarity in light mode
- [x] 5.5 Ensure grouped view (`GroupHeader.svelte`) matches refreshed visual system in both collapsed and expanded states

## 6. Interaction State Consistency

- [x] 6.1 Standardize hover/active/selected/disabled state styling across buttons, filter chips, list items, and cards
- [x] 6.2 Ensure keyboard focus-visible states are present and visually consistent across interactive controls
- [x] 6.3 Normalize loading, empty, and error visual feedback patterns in light mode
- [x] 6.4 Remove state-specific layout shifts (hover scaling/jumps) that reduce interaction stability

## 7. Accessibility and Motion Compliance

- [x] 7.1 Validate text contrast for primary and secondary text on all major surfaces in light mode
- [x] 7.2 Replace non-semantic clickable containers with semantic controls where required, or add correct ARIA and keyboard support
- [x] 7.3 Implement or confirm reduced-motion handling for non-essential transitions
- [x] 7.4 Review heading hierarchy and landmark semantics in main layout and dashboard page

## 8. Validation and Handoff

- [x] 8.1 Run `npm run check` and resolve new UI-related issues introduced by the refresh
- [x] 8.2 Run `npm run build` and verify production build output remains successful
- [ ] 8.3 Perform manual visual QA at key breakpoints (mobile/tablet/desktop) for layout consistency
- [ ] 8.4 Verify functional parity for existing workflows (search, filters, tagging, grouping, preview, git actions)
- [x] 8.5 Document dark-mode follow-up plan as a separate future change that reuses semantic tokens
