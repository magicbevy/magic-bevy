# MagicBevy - Project Overview

**MagicBevy** is a modular game engine built around a package-driven architecture, designed for lightweight workflows and AI-assisted game development.

## High-Level Summary

- **Language:** Rust (edition 2024)
- **GUI Framework:** eframe + egui (immediate mode GUI)
- **Build System:** Cargo workspace
- **License:** MIT
- **Primary Goal:** Hot-reloadable editor with dynamic package loading and a CLI package manager

## Workspace Members

| Crate | Path | Type | Purpose |
|-------|------|------|---------|
| `magic-bevy` (root) | `/` | binary/lib | Application entry point (`src/main.rs`), depends on `magic_core` |
| `magic_core` | `magic_core/` | library | Editor UI components and re-exports of the editor API |
| `magic_api_editor` | `magic_api_editor/` | library | Editor API: registries, windows, assets, compiler state |
| `magic_api_runtime` | `magic_api_runtime/` | library | Runtime API: render context, render driver, update loop |
| `magic_log` | `magic_log/` | library | Log store, log callback, and `mg*` / `oracle_*` macros |
| `magic_editor` | `magic_editor/` | cdylib | Editor dynamic library for hot-reload, depends on `magic_api_editor` + `magic_pkg_editor` |
| `magic_pkg` | `magic_pkg/` | binary/lib | CLI package manager, depends on `magic_log` |
| `magic_runtime` | `magic_runtime/` | library | Minimal runtime crate (`add` function + test) |
| `magic_pkg_editor` | `magic_editor/magic_pkg_editor/` | library | Auto-generated editor package registration (empty by default) |
| `magic_pkg_runtime` | `magic_runtime/magic_pkg_runtime/` | library | Auto-generated runtime package registration (empty by default) |

## Root `Cargo.toml` Dependencies

- `magic_core` (path dependency)
- `magic_pkg` (path dependency)
- `magic_log` (path dependency)
- `libloading` 0.8 — dynamic library loading for hot-reload
- `eframe` 0.36 — GUI framework
- `egui` 0.36 — immediate mode GUI
- `egui-phosphor` 0.14 — icon font
- `egui_dock` 0.21 — docking
- `notify` 8.1.0 — file system watcher

## `.cargo/config.toml`

- Linux target uses `clang` linker with `mold` linker flag
- `sccache` as rustc-wrapper for faster compilation
