## ADDED Requirements

### Requirement: Respect Exclusion Patterns during Discovery
The repository scanning engine SHALL apply exclusion filters to the traversal logic to reduce I/O and CPU usage.

#### Scenario: Scan with active filters
- **WHEN** a folder contains "node_modules" and "node_modules" is in the exclusion list
- **THEN** the scanner does not enter the "node_modules" directory at all

### Requirement: Depth-First Scanning with Constraint
The scanner SHALL implement a depth constraint that terminates traversal once the user-defined max depth is reached.

#### Scenario: Reach max depth
- **WHEN** scanning a directory at depth 5 and max depth is 4
- **THEN** the scan stops and returns only results found within depth 4
