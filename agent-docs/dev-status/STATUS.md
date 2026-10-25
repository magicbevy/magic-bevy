# Dev Status - MagicBevy

## Current Phase: Window System (non-docking implementation) + Code Refactoring

### ✅ Completed
- Initial codebase review and documentation (`agent-docs/`)
- Core menu registry restructured to hierarchical (recursive) model
  - `MenuItem` enum: `Action`, `Submenu`, `Separator`
  - `MenuEntry` with `items: Vec<MenuItem>`
  - `register_action(path, label, callback)` - supports unlimited depth
  - `register_submenu(path, label)` - creates submenus recursively
  - `register_in_core(core_menu, label, callback)` - adds to reserved core menus
  - `clear()` - keeps reserved menus, clears non-reserved
- `top_bar.rs` updated with recursive `draw_menu_items()` for unlimited nesting
- `default_menus.rs` created for core menu order (File, Edit, Window, Packages, View, Tools, Help)
- `src/main.rs` updated with new API (`register_action` instead of old `register`)
- `PackageManager::new(".").sync()` called on first launch before compile
- `magic_pkg` dependency added to root Cargo.toml
- `egui_dock` evaluated then removed (incompatible with egui 0.30/eframe 0.30)
- egui/eframe upgraded to 0.36, egui_dock 0.21 added
- Window system implemented without egui_dock (for now):
  - `WindowId`, `EditorWindow` trait, `WindowDescriptor`, `WindowFactory`, `WindowRegistry`
  - `WindowInstance`, `WindowInstances` for managing multiple open windows
  - `EditorRegistry` extended with `windows` + `open_window_tx`
  - `MagicBevyApp` holds `WindowRegistry`, `WindowInstances`, `window_open_tx/rx`
  - Windows rendered as floating `egui::Window`s in `update()`
  - `open_window()` creates instances from registered factories
  - `save_layout()` / `load_layout()` for TOML-based layout profiles
  - `list_layouts()` for enumerating saved layouts
- **Code refactored**: `main.rs` split into separate modules:
  - `src/main.rs` — entry point only (module declarations + `main()`)
  - `src/app.rs` — `MagicBevyApp` struct, `new()`, `ui()`, `reload_and_load_library()`, `mock_packages()`
  - `src/hot_reload.rs` — `CompilerMessage`, `execute_compile()`, `init_file_watcher()`
  - `src/windows.rs` — `WindowManager`, `open_window()`, `render_windows()`
  - `src/layouts.rs` — `LayoutManager`, `save_layout()`, `load_layout()`, `list_layouts()`
- **Top bar fixes (egui 0.36 migration):**
  - `Panel::top()` replaced with `Window` anchored `LEFT_TOP` with full width
  - `ui.close_menu()` → `ui.close()`
  - Removed `ctx.request_repaint()` from compiling indicator (stops infinite blink)
  - Solid blue LED instead of animated blink during compile
- All code compiles without warnings (`cargo check` passes)

### 🔄 In Progress
- None

### ⏳ Next Planned
- Integrate `egui_dock` for Unity-like wall docking (replacing floating windows)
- Wire up `save_layout` / `load_layout` to Window menu items
- Add rect/position saving for windows
- Wire `open_window` calls from menu actions to actual window opening
- Console/Debug Log Window
- Package Manager Window
- File Manager Window

### 🚫 Blocked
- None

### ⏳ Next Planned
- **Console/Debug Log Window**: Graphical console window similar to Unity's debug log — shows compiler messages, package sync output, and runtime logs with timestamps and severity levels (Info, Warning, Error)
- **Package Manager Window**: Graphical package management window — install/remove/enable/disable packages via GUI instead of CLI, view installed packages with versions, check for updates
- **File Manager Window**: Graphical file browser window — navigate project folders, open files, create/delete assets
- Wire up `save_layout` / `load_layout` to Window menu items
- Add rect/position saving for windows

### 🚫 Blocked
- None

---

## File Changes (current session)

| File | Change |
|------|--------|
| `magic_core/src/api/menu.rs` | Full rewrite for hierarchical menu structure |
| `magic_core/src/api/mod.rs` | Added `MenuItem` to re-exports |
| `magic_core/src/ui/top_bar.rs` | Recursive rendering for unlimited nesting |
| `magic_core/src/default_menus.rs` | Created with `CORE_MENU_ORDER` constant |
| `magic_core/src/lib.rs` | Added `default_menus` module; added `pub use egui` |
| `magic_core/src/api/editor.rs` | Added `windows` + `open_window_tx` fields to `EditorRegistry` |
| `magic_core/src/api/window.rs` | Created: `WindowId`, `EditorWindow`, `WindowDescriptor`, `WindowFactory`, `WindowRegistry`, `WindowInstance`, `WindowInstances` |
| `magic_core/Cargo.toml` | Changed `egui` from `0.33.3` to `0.30` (match eframe) |
| `Cargo.toml` | Added `magic_pkg` dependency; removed `egui_dock` |
| `src/main.rs` | Refactored to entry point only (module declarations + `main()`) |
| `src/app.rs` | New: `MagicBevyApp` struct, lifecycle, `eframe::App` impl |
| `src/hot_reload.rs` | New: `CompilerMessage`, compile logic, file watcher |
| `src/windows.rs` | New: `WindowManager`, window open/render |
| `src/layouts.rs` | New: `LayoutManager`, save/load/list layouts |

---

## Build Status
- `cargo check` — **PASS** (no warnings, no errors)
