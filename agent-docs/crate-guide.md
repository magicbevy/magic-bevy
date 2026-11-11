# Crate-by-Crate Guide

## `magic-bevy` (root crate)

**Path:** `/src/main.rs` (entry point) + `/src/app.rs`, `/src/hot_reload.rs`, `/src/windows.rs`, `/src/layouts.rs`  
**Type:** binary (eframe app)  
**Dependencies:** `magic_core`, `magic_pkg`, `magic_log`, `eframe`, `egui`, `notify`, `libloading`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `app` | `src/app.rs` | `MagicBevyApp` struct, `new()`, `ui()`, `reload_and_load_library()`, `mock_packages()`, `eframe::App` impl |
| `hot_reload` | `src/hot_reload.rs` | `CompilerMessage` enum, `execute_compile()`, `init_file_watcher()` |
| `windows` | `src/windows.rs` | `WindowManager`, `DockState<WindowTab>`, `WindowTabViewer`, `open_window()`, `save_layout_named()`, `load_layout_named()` |
| `layouts` | `src/layouts.rs` | `LayoutManager`, `save_layout()`, `load_layout()`, `list_layouts()` |

**Do not modify unless:** changing app entry point, window size, title, hot-reload logic, window management, layout save/load, or dock state.

---

## `magic_api_editor`

**Path:** `magic_api_editor/src/`
**Type:** library
**Dependencies:** `magic_log`, `eframe`, `egui`, `serde`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `api` | `api/mod.rs` | Re-exports editor registries and windows |
| `api/editor` | `api/editor.rs` | `EditorRegistry<'a>` — menu, events, windows, assets, log callback |
| `api/events` | `api/events.rs` | `EditorEventRegistry` — compile start/success callbacks |
| `api/menu` | `api/menu.rs` | `MenuRegistry` — hierarchical menu structure (Action, Submenu, Separator) |
| `api/window` | `api/window.rs` | `WindowId`, `EditorWindow` trait, `WindowDescriptor`, `WindowFactory`, `WindowRegistry`, `WindowInstances`, `WindowTab` |
| `api/assets` | `api/assets.rs` | `AssetTypeRegistry`, `AssetTypeDescriptor` |
| `compiler` | `compiler.rs` | `EngineCompilerState` global singleton (lock mode, reload request) |
| `status` | `status/mod.rs` | `CompilerStateManager`, `CompileStatus` |

**Do not modify unless:** changing editor package APIs or the compile state machine.

---

## `magic_api_runtime`

**Path:** `magic_api_runtime/src/`
**Type:** library
**Dependencies:** `magic_log`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `api/runtime` | `api/runtime.rs` | `RenderDriver`, `RenderContext`, `UpdateRegistry`, `RuntimeRegistry` |

**Do not modify unless:** changing runtime package APIs or the update loop.

---

## `magic_log`

**Path:** `magic_log/src/`
**Type:** library
**Dependencies:** `colored`, `chrono`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `log` | `log/mod.rs` | Log callback singleton and `push_log` |
| `log/store` | `log/store.rs` | `LogEntry`, `LogLevel`, `LogStore`, `global_store` |
| `log/macros` | `log/macros.rs` | `oracle_*` macros |
| `logger` | `logger.rs` | Colored `mg*` macros |
| `oracle` | `oracle.rs` | Convenience re-exports for `oracle_*` macros |

---

## `magic_core`

**Path:** `magic_core/src/`
**Type:** library
**Dependencies:** `magic_api_editor`, `magic_log`, `eframe`, `egui`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `lib` | `lib.rs` | Re-exports `magic_api_editor` and logging types/macros |
| `ui` | `ui/mod.rs` | Editor windows: top bar, indicator, Oracle Logs, Grimoire, inspector |

**Do not modify unless:** changing editor UI or the host-side integration.

---

## `magic_editor`

**Path:** `magic_editor/src/`
**Type:** cdylib (dynamic library)
**Dependencies:** `magic_api_editor`, `magic_log`, `magic_pkg_editor`

### Modules

| Module | File | Purpose |
|--------|------|---------|
| `lib` | `lib.rs` | Exports `register_all_packages` |
| `systems` | `systems/` | `EditorSystem` trait + `SystemManager` |
| `profile` | `profile/mod.rs` | Placeholder |

**Do not modify unless:** adding editor systems, changing the `EditorSystem` trait, or modifying the cdylib export symbol.

---

## `magic_pkg`

**Path:** `magic_pkg/src/`  
**Type:** library + binary (CLI)  
**Dependencies:** `clap`, `magic_log`, `serde`, `serde_json`, `toml`, `ureq`, `zip`

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
**Dependencies:** `magic_api_runtime`, `magic_log`, `magic_pkg_runtime`

Registers runtime packages and forwards the host log callback. Reserved for future runtime engine code.

### Sub-crate: `magic_pkg_runtime`

**Path:** `magic_runtime/magic_pkg_runtime/src/lib.rs`  
Auto-generated empty `register_all_packages`. Populated by `magic_pkg sync`.

---

## `magic_pkg_editor` (sub-crate)

**Path:** `magic_editor/magic_pkg_editor/src/lib.rs`  
Auto-generated empty `register_all_packages`. Populated by `magic_pkg sync` when editor packages exist.
