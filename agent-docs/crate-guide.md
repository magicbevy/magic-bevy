# Crate-by-Crate Guide

## `magic-bevy` (root crate)

**Path:** `/src/main.rs` (entry point) + `/src/app.rs`, `/src/hot_reload.rs`, `/src/windows.rs`, `/src/layouts.rs`  
**Type:** binary (eframe app)  
**Dependencies:** `magic_core`, `magic_pkg`, `eframe`, `egui`, `notify`, `libloading`, `colored`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `app` | `src/app.rs` | `MagicBevyApp` struct, `new()`, `ui()`, `reload_and_load_library()`, `mock_packages()`, `eframe::App` impl |
| `hot_reload` | `src/hot_reload.rs` | `CompilerMessage` enum, `execute_compile()`, `init_file_watcher()` |
| `windows` | `src/windows.rs` | `WindowManager`, `DockState<WindowTab>`, `WindowTabViewer`, `open_window()`, `save_layout_named()`, `load_layout_named()` |
| `layouts` | `src/layouts.rs` | `LayoutManager`, `save_layout()`, `load_layout()`, `list_layouts()` |

**Do not modify unless:** changing app entry point, window size, title, hot-reload logic, window management, layout save/load, or dock state.

---

## `magic_core`

**Path:** `magic_core/src/`  
**Type:** library  
**Dependencies:** `eframe`, `egui`, `colored`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `api` | `api/mod.rs` | Re-exports `EditorRegistry`, `EditorEventRegistry`, `MenuRegistry`, `RuntimeRegistry` |
| `api/editor` | `api/editor.rs` | `EditorRegistry<'a>` — menu + events references passed to packages |
| `api/events` | `api/events.rs` | `EditorEventRegistry` — compile start/success callbacks |
| `api/menu` | `api/menu.rs` | `MenuRegistry` — hierarchical menu structure (Action, Submenu, Separator) |
| `api/window` | `api/window.rs` | `WindowId`, `EditorWindow` trait, `WindowDescriptor`, `WindowFactory`, `WindowRegistry`, `WindowInstance`, `WindowInstances`, `WindowTab` |
| `api/runtime` | `api/runtime.rs` | `RuntimeRegistry<'a>` — minimal, future use |
| `compiler` | `compiler.rs` | `EngineCompilerState` global singleton (lock mode, reload request) |
| `logger` | `logger.rs` | Colored print macros (`mgprint!`, `mgwarn!`, `mgerror!`, `mgsuccess!`) |
| `status` | `status/mod.rs` | `CompilerStateManager`, `CompileStatus` enum |
| `status/compiler_state` | `status/compiler_state.rs` | `CompileStatus` variants + `CompilerStateManager` |
| `ui` | `ui/mod.rs` | Re-exports `top_bar`, `indicator` |
| `ui/top_bar` | `ui/top_bar.rs` | `draw_top_bar()` — egui top panel, returns `LayoutAction` for save/load |
| `ui/indicator` | `ui/indicator.rs` | `draw_status_indicator()` — blinking LED + text for compile status |
| `loader` | `loader/mod.rs` | `PackageManager` (editor lib loader), `PackageReloader` (legacy) |
| `loader/editor_lib` | `loader/editor_lib.rs` | Loads `libmagic_editor.so`, extracts `create_system` symbol |
| `loader/package_reloader` | `loader/package_reloader.rs` | Reads `package.toml` (legacy, not used in main flow) |
| `package_manager` | `package_manager/mod.rs` | Empty module |

**Do not modify unless:** changing core APIs, adding new engine-wide systems, or modifying the compile state machine.

---

## `magic_editor`

**Path:** `magic_editor/src/`  
**Type:** cdylib (dynamic library)  
**Dependencies:** `magic_core`, `magic_pkg_editor`, `eframe`, `egui`, `serde`, `serde_json`, `colored`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `lib` | `lib.rs` | Exports `register_all_packages` (delegates to `magic_pkg_editor`) |
| `systems` | `systems/mod.rs` | `EditorSystem` trait + `SystemManager` |
| `systems/system` | `systems/system.rs` | `EditorSystem` trait (`on_enable`, `on_update`, `on_disable`) |
| `systems/manager` | `systems/manager.rs` | `SystemManager` — owns `Vec<Box<dyn EditorSystem>>` |
| `profile` | `profile/mod.rs` | Commented-out placeholder |

**Do not modify unless:** adding editor systems, changing the `EditorSystem` trait, or modifying the cdylib export symbol.

---

## `magic_pkg`

**Path:** `magic_pkg/src/`  
**Type:** library + binary (CLI)  
**Dependencies:** `clap`, `magic_core`, `serde`, `serde_json`, `toml`, `ureq`, `zip`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `lib` | `lib.rs` | Module declarations + public re-exports |
| `main` | `main.rs` | CLI entry point, clap subcommand dispatch |
| `error` | `error.rs` | `PackageError` enum |
| `manifest` | `manifest.rs` | `PackageTarget`, `RawManifest`, `PackageManifest`, `ProjectPackagesManifest`, release types |
| `manager` | `manager.rs` | `PackageManager` — all package operations |
| `utils` | `utils.rs` | File ops, path helpers, manifest rewriting, table printing |

### Key Structs

- `PackageManager` — holds paths to workspace root, packages dir, editor/runtime manifests
- `SyncReport` — counters for discovered, enabled, editor, runtime, updated manifests/code files
- `PackageManifest` — parsed `package.toml` with category, vendor, name, version, enabled, targets, root
- `RawManifest` — serde deserialization of `package.toml`

**Do not modify unless:** changing package CLI commands, install/sync logic, or manifest formats.

---

## `magic_runtime`

**Path:** `magic_runtime/src/lib.rs`  
**Type:** library  
**Dependencies:** `magic_core`, `magic_pkg_runtime`

Minimal crate with `add()` function and test. Reserved for future runtime engine code.

### Sub-crate: `magic_pkg_runtime`

**Path:** `magic_runtime/magic_pkg_runtime/src/lib.rs`  
Auto-generated empty `register_all_packages`. Populated by `magic_pkg sync`.

---

## `magic_pkg_editor` (sub-crate)

**Path:** `magic_editor/magic_pkg_editor/src/lib.rs`  
Auto-generated empty `register_all_packages`. Populated by `magic_pkg sync` when editor packages exist.
