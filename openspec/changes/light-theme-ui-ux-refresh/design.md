## Context

Rust Harbor currently prioritizes a dark, highly stylized interface. While visually distinctive, the current presentation makes long-session scanning and reading harder, especially in dense repository lists, filters, and metadata-heavy cards. The app also lacks a clear semantic theming layer, so visual decisions are scattered across components instead of being driven by reusable tokens.

This change refreshes the UI/UX to a light-first system with stronger readability, clearer hierarchy, and consistent interaction patterns. The refresh must preserve existing functional behavior and keep future dark mode feasible through token-level theme switching rather than component rewrites.

## Goals / Non-Goals

**Goals:**
- Establish a light-first design token foundation for colors, text roles, spacing, radii, elevation, and motion.
- Increase readability with improved contrast, typography hierarchy, and content density balance.
- Standardize interaction states (hover, focus, active, selected, disabled, loading, empty, error) across navigation, cards, dialogs, and popovers.
- Improve accessibility baselines (contrast, keyboard focus visibility, semantic controls).
- Keep existing repository workflows and command behavior intact.
- Prepare architecture for future dark mode toggle by semantic token mapping.

**Non-Goals:**
- Implementing a user-facing dark mode toggle in this change.
- Redesigning backend architecture, Tauri command contracts, or database schema.
- Introducing major information architecture changes (new pages, new navigation model).
- Reworking product scope beyond UI/UX quality and consistency.

## Decisions

### Decision 1: Light-first semantic token system
Adopt semantic tokens (e.g., `--color-bg`, `--color-surface`, `--color-text-primary`, `--color-border-subtle`, `--color-accent`) in global styles and route-level components.

Rationale:
- Enables consistent styling and simpler maintenance.
- Creates a direct path to dark mode by remapping token values.
- Reduces ad-hoc hardcoded color usage.

Alternatives considered:
- Continue local component color tuning without token refactor.
Rejected because it perpetuates inconsistency and makes dark mode expensive.

### Decision 2: Readability-first typography system
Standardize type scale and usage for titles, section labels, metadata, and body text. Use a primary sans family for UI body with a restrained monospace role for paths/code-like metadata.

Rationale:
- Improves visual hierarchy in dense dashboard contexts.
- Reduces cognitive overhead from inconsistent sizes/weights.

Alternatives considered:
- Keep current mixed decorative typography.
Rejected due to low readability in high-frequency usage screens.

### Decision 3: Surface and state normalization
Define reusable surface tiers (page, panel, card, overlay), with state contracts:
- focus-visible always explicit
- hover feedback stable (no layout jump)
- selected states visually distinct and contrast-safe
- disabled states readable but clearly inactive

Rationale:
- Predictable state language reduces user friction.
- Improves discoverability for interactive elements.

Alternatives considered:
- Per-component bespoke state styling.
Rejected due to inconsistent behavior and quality drift.

### Decision 4: Progressive migration strategy (route-first)
Roll out refresh in highest-impact screens first:
1) global shell/layout
2) repository dashboard page
3) dialogs/popovers/panels
4) shared custom components

Rationale:
- Keeps review scope manageable.
- Allows incremental visual QA.

Alternatives considered:
- Big-bang full rewrite.
Rejected because risk and regression surface are higher.

## Risks / Trade-offs

- [Risk] Light palette may weaken brand identity from current dark aesthetic.
  → Mitigation: Keep accent personality via controlled highlight colors and distinctive typography treatment.

- [Risk] Contrast regressions in small metadata text.
  → Mitigation: Explicit contrast thresholds and targeted review for badges, tertiary labels, and disabled states.

- [Risk] Token migration misses isolated hardcoded colors.
  → Mitigation: Search-and-audit pass for color literals in major UI files; enforce semantic token usage in touched components.

- [Risk] Animation refinements may conflict with accessibility preferences.
  → Mitigation: Keep motion subtle and gate non-essential transitions for `prefers-reduced-motion`.

- [Trade-off] Initial refactor cost increases before feature work.
  → Mitigation: This establishes a stable UI foundation that accelerates future features, including dark mode toggle.

## Migration Plan

1. Introduce semantic tokens in global style layer.
2. Map existing core components to semantic tokens.
3. Refresh layout shell and repository dashboard visuals.
4. Align popovers, dialogs, and side panels to the same state/spacing contract.
5. Run build and accessibility smoke checks.
6. Document follow-up dark mode mapping plan as next change.

Rollback strategy:
- Keep token definitions and component deltas in isolated commits to revert quickly.
- If severe regressions occur, revert style-layer commit first, then route-level component commits.

## Open Questions

- Should the product keep a single accent color or allow per-context accent variants (navigation vs status vs actions)?
- Do we want font self-hosting in this phase, or continue remote font loading and optimize later?
- What exact visual density target (compact vs comfortable) should be default for repository list rows?
