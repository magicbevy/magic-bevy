# MagicBevy Agent Documentation Index

This folder contains documentation for AI agents working on the MagicBevy project. Read these files instead of reading the entire codebase on every session.

## Files

| File | Description |
|------|-------------|
| `project-overview.md` | High-level project summary, workspace members, dependencies |
| `architecture.md` | Detailed architecture: app lifecycle, hot-reload flow, core APIs, data flow |
| `crate-guide.md` | Per-crate guide: purpose, modules, key files, what to modify |
| `workflows.md` | Step-by-step workflows: hot-reload, package init/sync/install/update/bump |
| `key-files.md` | Quick reference to all important files and exported symbols |
| `coding-conventions.md` | Naming, formatting, logging, error handling, concurrency, unsafe rules |
| `dev-status/STATUS.md` | Current dev phase, completed/in-progress items, file changes, build status |

## How to Use

1. **New to the project?** Start with `project-overview.md`, then `architecture.md`.
2. **Need to change a specific crate?** Read `crate-guide.md` for that crate.
3. **Implementing a feature?** Read `workflows.md` for the relevant flow.
4. **Need a file path?** Check `key-files.md`.
5. **Writing code?** Follow `coding-conventions.md`.

## Project Facts (Quick Reference)

- **Root entry point:** `src/main.rs` (`MagicBevyApp`, eframe/egui)
- **Core library:** `magic_core/src/lib.rs`
- **Editor cdylib:** `magic_editor/src/lib.rs` (exports `register_all_packages`)
- **Package manager CLI:** `magic_pkg/src/main.rs` (clap subcommands)
- **Package manifest:** `packages/packages.toml`
- **Build config:** `.cargo/config.toml` (clang + mold, sccache)
- **Hot-reload:** file watcher → `cargo check/build -p magic_editor` → `libloading` → `register_all_packages`

## Common Tasks

| Task | Where to look |
|------|---------------|
| Change top bar height / dock controls | `magic_core/src/ui/top_bar.rs` — `ui.set_min_height()`, `draw_top_bar()` returns `LayoutAction` |
| Add dockable window | `magic_core/src/api/window.rs` — `WindowRegistry::register()` + `WindowTab` |
| Save/load layout | `src/windows.rs` — `save_layout_named()`, `load_layout_named()` |
| Hot-reload on file change | `src/hot_reload.rs` — `init_file_watcher()`, `execute_compile()` |
| Add editor system | `magic_editor/src/systems/system.rs` + `manager.rs` |
| Change package manifest format | `magic_pkg/src/manifest.rs` |
| Add new CLI subcommand | `magic_pkg/src/main.rs` + `magic_pkg/src/manager.rs` |
