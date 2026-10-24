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
| `magic_core` | `magic_core/` | library | Core engine: APIs, compiler state, logger, UI components, loader |
| `magic_editor` | `magic_editor/` | cdylib | Editor dynamic library for hot-reload, depends on `magic_core` + `magic_pkg_editor` |
| `magic_pkg` | `magic_pkg/` | binary/lib | CLI package manager, depends on `magic_core` |
| `magic_runtime` | `magic_runtime/` | library | Minimal runtime crate (`add` function + test) |
| `magic_pkg_editor` | `magic_editor/magic_pkg_editor/` | library | Auto-generated editor package registration (empty by default) |
| `magic_pkg_runtime` | `magic_runtime/magic_pkg_runtime/` | library | Auto-generated runtime package registration (empty by default) |

## Root `Cargo.toml` Dependencies

- `magic_core` (path dependency)
- `libloading` 0.8 — dynamic library loading for hot-reload
- `eframe` 0.30 — GUI framework
- `egui` 0.33.3 — immediate mode GUI
- `notify` 8.1.0 — file system watcher
- `colored` 3.1.1 — colored terminal output

## `.cargo/config.toml`

- Linux target uses `clang` linker with `mold` linker flag
- `sccache` as rustc-wrapper for faster compilation
