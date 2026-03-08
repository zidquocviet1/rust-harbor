## Context

The current repository dashboard provides a basic overview:
- One primary language per repository.
- A remote connectivity check that only verifies if a remote URL exists.
- A "Refresh" button that re-triggers a full scan.
- No visibility into repository descriptions or content (README) without leaving the app.
- A grid/list toggle, but the list view is under-optimized for frequent Git operations.

## Goals / Non-Goals

**Goals:**
- **Rich Metadata**: Detect and weight multiple programming languages per repository.
- **Interactive Inspection**: Peek into repositories (README) without navigating away using a side-sheet.
- **Operational Efficiency**: Prioritize 'Fetch' and 'Pull' as primary actions in high-density views.
- **System Realism**: Provide accurate remote accessibility status using lightweight network probing.
- **Visual Clarity**: Add tooltips/labels to actions and a refined, frequency-sorted filter bar.

**Non-Goals:**
- Full-blown source code editor (inspection only).
- Multi-remote management (focus on 'origin').
- Complex Git operations like rebase or conflict resolution (stay within fetch/pull/push).

## Decisions

### 1. Multi-Language Detection Strategy
We will move away from single-extension counting to a weighted analysis.
- **Rationale**: Real-world projects are polyglot. A "Rust" project often has significant JS/TS (frontend) or YAML (CI/CD).
- **Implementation**: Scan all files (respecting depth limits and pruning), count by extension, and return a dictionary of `{ Language: Percentage }`.
- **Alternatives**: Using the `linguist` library was considered but rejected due to heavy dependencies; custom extension-based weighting in Rust is faster and sufficiently accurate for this use case.

### 2. Live Connectivity Verification
Instead of just checking for the existence of a URL, we will use a lightweight `git ls-remote` probe.
- **Rationale**: A repository might have a valid URL but be inaccessible due to auth expiration, network issues, or deletion.
- **Implementation**: Run `git ls-remote --exit-code --heads origin` with a 5-second timeout.
- **Trade-off**: Increases scan time slightly. Mitigation: Perform this check only during "Refresh" or on explicit hover/select, rather than every background sync.

### 3. Integrated README Side-Panel
A sliding side-sheet (`Drawer` or `Sheet` pattern) will display repository content.
- **Rationale**: Allows users to quickly confirm a repo's purpose or setup without context switching.
- **Implementation**: 
    - Use Svelte's `transition:fly` or `transition:slide` for the entrance animation.
    - Support three modes: `Markdown` (rendered), `Plain Text` (raw), and `Unified` (header + metadata summary).
- **Library Choice**: Use `marked` for Markdown parsing and `dompurify` (or equivalent) for safe rendering.

### 4. Direct Action Optimization (List View)
Swap "Open" and "Refresh" for "Fetch" and "Pull" in the compact list view.
- **Rationale**: The list view is for power users managing many repos. They need to keep things in sync quickly.
- **Action Behavior**: 'Pull' will default to the current checked-out branch only to avoid complex merge/rebase prompts.

## Risks / Trade-offs

- **[Risk] Network Latency** → Probing many remotes simultaneously could hang. *Mitigation*: Use a short per-command timeout (5s) and limit concurrency in the background task.
- **[Risk] Large READMEs** → Loading massive files into the preview might freeze the UI. *Mitigation*: Read only the first 50KB of README files for preview.
- **[Risk] Filter Bar Overflow** → 20+ languages in the dock will break the layout. *Mitigation*: Frequency-sorted sorting + horizontal scroll + "+x" dropdown for low-frequency languages.
