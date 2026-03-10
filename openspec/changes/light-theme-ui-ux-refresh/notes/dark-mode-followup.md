## Dark Mode Follow-up Plan

This refresh establishes semantic light-first tokens. Dark mode should be implemented as a separate change by remapping tokens only.

### Proposed Change Name
- `theme-toggle-dark-mode`

### Goals
- Add user-facing theme toggle (Light / Dark / System)
- Persist preference in app config
- Keep component markup unchanged

### Strategy
1. Introduce theme state (`light | dark | system`) in frontend store.
2. Persist selection via config command and apply on app boot.
3. Define dark token equivalents for all semantic tokens in `src/app.css`.
4. Avoid per-component hardcoded dark classes; rely on semantic variables.
5. Run contrast checks for both themes and include reduced-motion compatibility.

### Validation Checklist
- Theme switch updates all major routes and overlays.
- Preference survives app restart.
- Search/filter/tag/grouping interactions visually stable in both modes.
- No readability regression on metadata-heavy cards/lists.
