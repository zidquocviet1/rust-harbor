## ADDED Requirements

### Requirement: Weighted Language Analysis
The system must analyze all files in the repository to determine the weight of each programming language used, rather than just the primary one.

#### Scenario: Balanced Polyglot Project
- **WHEN** A repository contains both Svelte (40%) and Rust (60%) files.
- **THEN** The backend must return both languages with their respective percentages or counts.

### Requirement: Optimized Language Filter
The dashboard filter bar must automatically sort languages by their frequency across all watched repositories and handle high-cardinality via a more/overflow menu.

#### Scenario: High Cardinality Management
- **WHEN** More than 8 distinct languages are detected across the dock.
- **THEN** The filter bar shows the top 7, followed by a "+x" button that opens a dropdown for the remaining languages.
