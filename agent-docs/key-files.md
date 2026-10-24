# Key Files Reference

## Root Level

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace definition + root crate deps (libloading, eframe, egui, notify, colored) |
| `Cargo.lock` | Locked dependency tree (auto-generated) |
| `src/main.rs` | Application entry point, hot-reload logic, file watcher, UI |
| `packages/packages.toml` | Project package manifest (dependencies map) |
| `.cargo/config.toml` | Linker (clang + mold), rustc-wrapper (sccache) |
| `.gitignore` | Git ignore rules (target, debug, .rs.bk, *.pdb, mutants.out) |
| `LICENSE` | MIT License |
| `README.md` | Brief project description |

## `magic_core/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Core crate deps (eframe, egui, colored) |
| `src/lib.rs` | Module declarations + re-exports |
| `src/compiler.rs` | `EngineCompilerState` global singleton (lock mode, reload request) |
| `src/logger.rs` | Colored print macros |
| `src/api/mod.rs` | API module re-exports |
| `src/api/editor.rs` | `EditorRegistry` struct |
| `src/api/events.rs` | `EditorEventRegistry` (compile callbacks) |
| `src/api/menu.rs` | `MenuRegistry` + `MenuCommand` |
| `src/api/runtime.rs` | `RuntimeRegistry` (minimal) |
| `src/status/compiler_state.rs` | `CompileStatus` enum + `CompilerStateManager` |
| `src/ui/top_bar.rs` | `draw_top_bar()` — egui top panel |
| `src/ui/indicator.rs` | `draw_status_indicator()` — LED + text |
| `src/loader/editor_lib.rs` | `PackageManager` — loads editor cdylib |
| `src/loader/package_reloader.rs` | `PackageReloader` — reads `package.toml` (legacy) |
| `src/package_manager/mod.rs` | Empty |

## `magic_editor/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | cdylib crate, deps on magic_core, magic_pkg_editor, eframe, egui, serde, colored |
| `src/lib.rs` | Exports `register_all_packages` (no_mangle, extern "Rust") |
| `src/systems/system.rs` | `EditorSystem` trait |
| `src/systems/manager.rs` | `SystemManager` — owns editor systems |
| `src/profile/mod.rs` | Commented placeholder |
| `src/magic_pkg_editor/src/lib.rs` | Auto-generated empty `register_all_packages` |

## `magic_pkg/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | CLI crate, deps (clap, magic_core, serde, toml, ureq, zip) |
| `src/lib.rs` | Module declarations + re-exports |
| `src/main.rs` | CLI entry point, all subcommand handlers |
| `src/error.rs` | `PackageError` enum |
| `src/manifest.rs` | `PackageTarget`, `RawManifest`, `PackageManifest`, release types |
| `src/manager.rs` | `PackageManager` — init, install, remove, sync, bump, status |
| `src/utils.rs` | File ops, manifest rewriting, lib.rs generation, table printing |

## `magic_runtime/`

| File | Purpose |
|------|---------|
| `Cargo.toml` | Runtime crate deps |
| `src/lib.rs` | `add()` function + test |
| `magic_pkg_runtime/src/lib.rs` | Auto-generated empty `register_all_packages` |

## Symbols Exported

| Symbol | Location | Purpose |
|--------|----------|---------|
| `register_all_packages` | `magic_editor/src/lib.rs:9` | cdylib export, called by hot-reload loader |
| `register_all_packages` | `magic_editor/magic_pkg_editor/src/lib.rs:4` | Auto-generated, calls editor package registers |
| `register_all_packages` | `magic_runtime/magic_pkg_runtime/src/lib.rs:4` | Auto-generated, calls runtime package registers |
| `create_system` | Not currently exported | Referenced in `magic_core/src/loader/editor_lib.rs:35` but not produced by `magic_editor` |

## Common Patterns

- Colored logging: `mgprint!("Tag", "message")`, `mgsuccess!`, `mgwarn!`, `mgerror!`
- Compile status: `CompilerStateManager` with `CompileStatus::Idle` / `Compiling` / `Success` / `Error`
- Global compiler state: `EngineCompilerState::global()` (OnceLock singleton)
- Package ID format: `vendor/name`
- Cargo alias format: `package_<vendor>_<name>` (cargo_identifier sanitizes)
- Directory layout: `packages/<category>/<vendor>/<name>/<editor|runtime>/`
