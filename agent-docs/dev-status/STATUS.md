# Dev Status - MagicBevy

## Current Phase: Window/Dock System (egui_dock integration complete)

### ✅ Completed
- Window/Dock system fully implemented with egui_dock
- Layout management: save, load, delete with project + global directories
- Layout draft auto-save (`.temp.layout`) and restore on startup

### 🐛 Bug Fixes
- Dock window disappearing: synced `WindowInstances` with `DockState` tabs
- Bottom panel occlusion: removed `CentralPanel` covering docked panels
- Layout selection not updating in ComboBox: tracks `current_layout` state
- egui ID conflict when loading layouts: rebuilds window instances
- Duplicate layout entry: removed `+` button, replaced with `💾 Save`

### 🔄 In Progress
- None

### ⏳ Next Planned
- Console/Debug Log Window
- Package Manager Window
- File Manager Window

### 🚫 Blocked
- None

---

## File Changes (current session)

| File | Change |
|------|--------|
| `magic_core/src/ui/top_bar.rs` | Fixed ComboBox selection tracking; removed `+` button label; Save/Delete windows anchored right |
| `src/app.rs` | Added `current_layout_name` field; startup loads temp layout then active layout |
| `src/windows.rs` | Added `rebuild_instances_for_layout()` and `load_temp_layout()`; auto-saves `.temp.layout` draft |
| `src/layouts.rs` | Moved `active_layout.json` to `./layouts/`; added temp layout persistence methods |

---

## Build Status
- `cargo check` — **PASS** (no warnings, no errors)
