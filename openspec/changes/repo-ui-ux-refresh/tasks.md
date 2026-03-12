## 1. Branding

- [x] 1.1 Identify all UI locations that display app name/icon (sidebar, header, window title)
- [x] 1.2 Add a single branding config source and wire name/icon consumers to it
- [x] 1.3 Replace existing hard-coded name/icon usages

## 2. Sidebar Navigation Simplification

- [x] 2.1 Locate folder list UI and remove detailed path text rendering
- [x] 2.2 Verify folder items still show icon and folder name clearly

## 3. View Mode Adjustments

- [x] 3.1 Remove raw view entry point from folder open flow
- [x] 3.2 Update unified view to render tag chips/badges
- [x] 3.3 Render remote URLs as hyperlinks with consistent link styling

## 4. Repository Detail Performance

- [x] 4.1 Identify heavy resources in repo detail (images/large assets)
- [x] 4.2 Implement lazy-loading for offscreen images
- [x] 4.3 Add placeholders and defer non-critical loading to avoid UI stutter
- [x] 4.4 Verify initial layout renders before heavy resources complete loading
