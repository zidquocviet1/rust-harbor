## ADDED Requirements

### Requirement: Semantic light-theme tokens
The system SHALL define and use semantic theme tokens for all core UI color roles in light mode, including background, surface, border, text, accent, and status roles.

#### Scenario: Page and surface rendering in light mode
- **WHEN** the application renders layout containers, cards, panels, dialogs, and popovers
- **THEN** each surface uses semantic light-theme tokens instead of hardcoded per-component color literals

#### Scenario: Future theme portability
- **WHEN** a future theme variant (e.g., dark mode) is introduced
- **THEN** visual adaptation can be achieved primarily by remapping token values without rewriting component structure

### Requirement: Light-mode readability and contrast baseline
The system SHALL provide accessible text contrast and hierarchy in light mode for primary, secondary, and metadata text.

#### Scenario: Primary content readability
- **WHEN** body content and primary labels are shown on light surfaces
- **THEN** text contrast remains sufficient for sustained reading and scanning, with clear distinction from secondary text

#### Scenario: Metadata legibility
- **WHEN** paths, badges, counts, and timestamps are displayed
- **THEN** metadata text remains readable and visually subordinate to primary content without becoming faint

### Requirement: Consistent interactive state language
The system SHALL standardize hover, focus-visible, active, selected, disabled, loading, and empty state semantics across interactive controls.

#### Scenario: Keyboard navigation visibility
- **WHEN** a user navigates interactive controls via keyboard
- **THEN** focus-visible states are consistently present and visually obvious in light mode

#### Scenario: Selected and active states
- **WHEN** a control or filter is selected/active
- **THEN** selected styling is clearly distinguishable from default and hover states using semantic state tokens

### Requirement: Motion accessibility baseline
The system SHALL use subtle motion for transitions and respect reduced-motion preferences.

#### Scenario: Standard transitions
- **WHEN** panels, menus, or groups animate between states
- **THEN** transitions use short, non-disorienting durations and do not cause layout instability

#### Scenario: Reduced-motion preference
- **WHEN** the user has reduced-motion enabled at OS/browser level
- **THEN** non-essential animations are reduced or disabled while preserving functional clarity
