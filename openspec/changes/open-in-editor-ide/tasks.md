## 1. Backend Models and Logic

- [x] 1.1 Create `EditorInfo` struct in `src-tauri/src/models/repo.rs` or a new model file.
- [x] 1.2 Implement `editor_service` to list supported editors and check their installation status on macOS (Bundle ID), Windows (Registry/PATH), and Linux (PATH/Desktop files).
- [x] 1.3 Add a list of well-known editors (VS Code, Cursor, Zed, IntelliJ, etc.) with their bundle IDs and display names.
- [x] 1.4 Implement a function to open a directory path in a specific editor using `Command`.

## 2. Tauri Commands and Controllers

- [x] 2.1 Create `get_installed_editors` command in `src-tauri/src/controllers/repo.rs` (or a specific editor controller).
- [x] 2.2 Create `open_in_editor` command to trigger the editor opening logic.
- [x] 2.3 Register new commands in `src-tauri/src/lib.rs`.

## 3. Frontend UI Implementation

- [x] 3.1 Create a new Sidebar/Panel component to display the "Open In" options.
- [x] 3.2 Update the Repository Detailed panel to include an "Open In" button with a dropdown of discovered editors.
- [x] 3.3 Add "Open In" options to the repository list context menu.
- [x] 3.4 Ensure the UI only shows editors returned by the backend.

## 4. Testing and Polishing

- [x] 4.1 Verify that the "Open In" menu updates correctly based on installed applications.
- [x] 4.2 Test opening repositories in at least 2 different installed editors.
- [x] 4.3 Add loading states while detecting editors if necessary.
