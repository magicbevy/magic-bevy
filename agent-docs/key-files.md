# Key Files Reference

## Root Level

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace definition + root crate deps (libloading, eframe, egui, notify, colored) |
| `Cargo.lock` | Locked dependency tree (auto-generated) |
| `src/main.rs` | Application entry point (module declarations + `main()`) |
| `src/app.rs` | `MagicBevyApp` struct, lifecycle, `eframe::App` impl |
| `src/hot_reload.rs` | `CompilerMessage`, compile logic, file watcher |
| `src/windows.rs` | `WindowManager`, window open/render |
| `src/layouts.rs` | `LayoutManager`, save/load/list layouts |
| `packages/packages.toml` | Project package manifest (dependencies map) |
| `.cargo/config.toml` | Linker (clang + mold), rustc-wrapper (sccache) |
| `.gitignore` | Git ignore rules (target, debug, .rs.bk, *.pdb, mutants.out) |
| `LICENSE` | MIT License |
| `README.md` | Brief project description |

## `magic_api_editor/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Editor API crate, deps on magic_log, eframe, egui, serde |
| `src/lib.rs` | Module declarations + re-exports |
| `src/compiler.rs` | `EngineCompilerState` global singleton |
| `src/api/editor.rs` | `EditorRegistry` struct |
| `src/api/events.rs` | `EditorEventRegistry` (compile callbacks) |
| `src/api/menu.rs` | `MenuRegistry` + `MenuItem` + `MenuEntry` |
| `src/api/window.rs` | `WindowId`, `EditorWindow`, `WindowRegistry`, `WindowInstances`, `WindowTab` |
| `src/api/assets.rs` | `AssetTypeRegistry`, `AssetTypeDescriptor` |
| `src/status/compiler_state.rs` | `CompileStatus` enum + `CompilerStateManager` |

## `magic_api_runtime/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Runtime API crate, dep on magic_log |
| `src/lib.rs` | Runtime API re-exports |
| `src/api/runtime.rs` | `RenderDriver`, `RenderContext`, `UpdateRegistry`, `RuntimeRegistry` |

## `magic_log/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Logging crate, deps on colored + chrono |
| `src/lib.rs` | Log callback, re-exports |
| `src/log/store.rs` | `LogEntry`, `LogLevel`, `LogStore`, `global_store` |
| `src/log/macros.rs` | `oracle_*` macros |
| `src/logger.rs` | Colored `mg*` macros |

## `magic_core/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Editor UI crate, deps on magic_api_editor, magic_log, eframe, egui |
| `src/lib.rs` | Re-exports editor API + logging |
| `src/ui/top_bar.rs` | `draw_top_bar()` — egui top panel with dock controls |
| `src/ui/indicator.rs` | `draw_status_indicator()` — LED + text |
| `src/ui/oracle_logs.rs` | Oracle Logs window |
| `src/ui/grimoire/` | Unified file/package manager window |
| `src/ui/inspector.rs` | Inspector window |
| `src/package_manager/mod.rs` | Empty |

## `magic_editor/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | cdylib crate, deps on magic_api_editor, magic_log, magic_pkg_editor |
| `src/lib.rs` | Exports `register_all_packages` (no_mangle, extern "Rust") |
| `src/systems/system.rs` | `EditorSystem` trait |
| `src/systems/manager.rs` | `SystemManager` — owns editor systems |
| `src/profile/mod.rs` | Commented placeholder |

## `magic_pkg/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | CLI crate, deps (clap, magic_log, serde, toml, ureq, zip) |
| `src/lib.rs` | Module declarations + re-exports |
| `src/main.rs` | CLI entry point, all subcommand handlers |
| `src/error.rs` | `PackageError` enum |
| `src/manifest.rs` | `PackageTarget`, `RawManifest`, `PackageManifest`, release types |
| `src/manager.rs` | `PackageManager` — init, install, remove, sync, bump, status |
| `src/utils.rs` | File ops, manifest rewriting, lib.rs generation, table printing |

## `magic_runtime/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Runtime crate deps (magic_api_runtime, magic_log) |
| `src/lib.rs` | Runtime package registration + log callback forwarding |
| `src/main.rs` | Runtime executable loop |
| `magic_pkg_runtime/src/lib.rs` | Auto-generated `register_all_packages` |

## Symbols Exported

| Symbol | Location | Purpose |
|--------|----------|---------|
| `register_all_packages` | `magic_editor/src/lib.rs` | cdylib export, called by hot-reload loader |
| `register_all_packages` | `magic_editor/magic_pkg_editor/src/lib.rs` | Auto-generated, calls editor package registers |
| `register_all_packages` | `magic_runtime/src/lib.rs` | Runtime export, called by runtime packages |
| `register_all_packages` | `magic_runtime/magic_pkg_runtime/src/lib.rs` | Auto-generated, calls runtime package registers |

## Common Patterns

- Colored logging: `magic_log::mgprint!("Tag", "message")`, `mgsuccess!`, `mgwarn!`, `mgerror!`
- Compile status: `CompilerStateManager` with `CompileStatus::Idle` / `Compiling` / `Success` / `Error`
- Global compiler state: `EngineCompilerState::global()` (OnceLock singleton)
- Package ID format: `vendor/name`
- Cargo alias format: `package_<vendor>_<name>` (cargo_identifier sanitizes)
- Directory layout: `packages/<category>/<vendor>/<name>/<editor|runtime>/`
