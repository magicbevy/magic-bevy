# Architecture Guide

## 1. Application Entry Point (`src/main.rs`)

The root crate `magic-bevy` contains `src/main.rs` which is the eframe/egui application entry point.

### `MagicBevyApp` struct

Contains:
- `menu_registry: MenuRegistry` — top-bar menu items
- `editor_events: EditorEventRegistry` — compile event callbacks
- `compiler_manager: CompilerStateManager` — compile status (Idle/Compiling/Success/Error)
- `tx` / `rx` — mpsc channel for compiler messages
- `is_compiling: Arc<Mutex<bool>>` — compilation lock flag
- `_watcher: notify::RecommendedWatcher` — file system watcher
- `current_lib: Option<libloading::Library>` — currently loaded editor dynamic library

### Lifecycle

1. `new(cc)` — initializes dark theme, creates channel, starts file watcher, calls `reload_and_load_library()`, unlocks watcher via `EngineCompilerState::global().unlock_watcher()`
2. `update(ctx, _frame)` — eframe `App` trait impl. Checks `request_reload` flag, processes compiler messages from channel, draws top bar and central panel
3. `main()` — creates `packages` directory, runs `eframe::run_native` with title "MagicBevy", size 1280x720

### Hot-Reload Flow

1. File watcher monitors `./packages` and `./magic_editor` for `.rs`, `Cargo.toml`, `Cargo.lock`, `package.toml` changes (ignores `target/` and `.git/`)
2. On valid change, sets `pending = true` and records `Instant::now()`
3. Background thread loops every 200ms; if `pending && !compiling && 1.5s elapsed`, calls `execute_compile()`
4. `execute_compile()` runs `cargo check -p magic_editor --message-format=short`, then `cargo build -p magic_editor --lib`
5. On success, sends `CompilerMessage::CompileSuccess` → `reload_and_load_library()` reloads the `.so`/`.dll`/`.dylib` and calls `register_all_packages`
6. The loaded library exports `register_all_packages(registry: &mut EditorRegistry)` via `#[no_mangle] extern "Rust"`

### `CompilerMessage` enum

- `Indexing(String, usize)` — progress message (file name, count)
- `CompileSuccess` — compilation succeeded
- `CompileError(String)` — compilation failed

---

## 2. `magic_core` — Core Library

Exposed modules (re-exported from `lib.rs`):
- `api` — `EditorRegistry`, `EditorEventRegistry`, `MenuRegistry`, `RuntimeRegistry`
- `status` — `CompilerStateManager`, `CompileStatus`
- `compiler` — `CompilerLockMode`, `EngineCompilerState`
- `ui` — `draw_top_bar` (from `top_bar`)
- `package_manager` — empty module
- `logger` — `mgprint!`, `mgwarn!`, `mgerror!`, `mgsuccess!` macros

### `EngineCompilerState` (`magic_core/src/compiler.rs`)

Global singleton via `OnceLock`. Fields:
- `lock_mode: Mutex<CompilerLockMode>` — `Free` or `EditorLock`. When `EditorLock`, file watcher ignores changes
- `request_reload: Mutex<bool>` — when `true`, triggers recompile on next UI update

Methods:
- `lock_watcher()` / `unlock_watcher()` — toggle lock mode
- `trigger_recompile()` — set `request_reload = true`

### `CompilerStateManager` (`magic_core/src/status/compiler_state.rs`)

Holds `CompileStatus`:
- `Idle`
- `Compiling { current_file: String, progress: usize }`
- `Success`
- `Error(String)`

Methods: `set_compiling(file, progress)`, `set_success()`, `set_error(err)`

### `EditorRegistry` (`magic_core/src/api/editor.rs`)

```rust
pub struct EditorRegistry<'a> {
    pub menu: &'a mut MenuRegistry,
    pub events: &'a mut EditorEventRegistry,
}
```

Passed to `register_all_packages` so packages can register menu items and event callbacks.

### `MenuRegistry` (`magic_core/src/api/menu.rs`)

- `menus: BTreeMap<String, Vec<MenuCommand>>`
- `register(menu_name, label, callback)` — adds a command under a menu name
- `clear()` — clears all menus

### `EditorEventRegistry` (`magic_core/src/api/events.rs`)

- `on_compile_start: Vec<EventCallback>`
- `on_compile_success: Vec<EventCallback>`
- `listen_compile_start(F)`, `listen_compile_success(F)`

### `RuntimeRegistry` (`magic_core/src/api/runtime.rs`)

Minimal: `pub data: &'a str`. Reserved for future runtime package registration.

### UI Components

- `ui/top_bar.rs` — `draw_top_bar(ctx, registry, compile_status)` — renders `TopBottomPanel` with menu buttons and status indicator on the right
- `ui/indicator.rs` — `draw_status_indicator(ctx, ui, status)` — draws LED circle (gray=Idle, blinking blue=Compiling, green=Success, red=Error)
- `logger.rs` — colored macros: tag in bright color, message in corresponding color (blue/red/yellow/green)

### Loader (`magic_core/src/loader/`)

- `editor_lib.rs` — `PackageManager` struct with `reload_editor()` that loads `libmagic_editor.so` and extracts `create_system` symbol (note: this symbol is NOT currently used by `magic_editor`, which exports `register_all_packages` instead)
- `package_reloader.rs` — `PackageReloader` — reads `package.toml` manifests (legacy/unused path)

---

## 3. `magic_editor` — Editor Dynamic Library

Crate type: `cdylib`. Output: `libmagic_editor.so` (Linux), `magic_editor.dll` (Windows), `libmagic_editor.dylib` (macOS).

### `src/lib.rs`

Exports `#[no_mangle] pub extern "Rust" fn register_all_packages(registry: &mut EditorRegistry)` which delegates to `magic_pkg_editor::register_all_packages(registry)`.

### `src/systems/system.rs`

Trait `EditorSystem: Send + Sync`:
- `on_enable(&mut self)`
- `on_update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui)`
- `on_disable(&mut self)`

### `src/systems/manager.rs`

`SystemManager` holds `Vec<Box<dyn EditorSystem>>`. Calls `on_enable` on add, `on_disable` on reload and in `Drop`.

### `src/profile/mod.rs`

Commented-out placeholder.

### `src/magic_pkg_editor/src/lib.rs` (auto-generated)

Empty `register_all_packages` function. Populated by `magic_pkg sync` when editor packages exist.

---

## 4. `magic_pkg` — CLI Package Manager

Binary crate with clap subcommands.

### Subcommands

| Command | Description |
|---------|-------------|
| `Init` | Create a new package (`packages/<category>/<vendor>/<name>/`) with `package.toml` and target templates |
| `Install` | Install from zip file or `vendor/name[#version]` repo reference |
| `Remove` | Remove a package and clean up empty dirs |
| `Enable` / `Disable` | Set `status` field in `package.toml` |
| `Outdated` | Check for newer compatible versions |
| `Update` | Update one or all packages |
| `Versions` | List remote releases for a package |
| `Bump` | Update `versions.toml` with current timestamp and magic version |
| `Sync` | Sync dependencies into `magic_pkg_editor` / `magic_pkg_runtime` Cargo.toml and `lib.rs` |
| `List` | List discovered packages in table format |

### `PackageManager` (`magic_pkg/src/manager.rs`)

Key methods:
- `new(start_path)` — finds workspace root, sets up paths to `packages/`, `magic_pkg_editor`, `magic_pkg_runtime`
- `discover()` — walks `packages/<category>/<vendor>/<name>`, reads `package.toml`, returns `Vec<PackageManifest>`
- `sync()` — installs missing deps, updates `Cargo.toml` `[dependencies]` with `package_*` aliases, updates `lib.rs` registration files
- `init_package(category, vendor, name, targets)` — creates directory structure, `package.toml`, target crates, calls `sync()`
- `install_repo(vendor, name, version)` — resolves version, downloads from GitHub tags, installs from zip
- `install_from_zip(zip_path)` — extracts zip, copies to `packages/<category>/<vendor>/<name>/`, updates project manifest
- `set_package_status(vendor, name, enabled)` — toggles `status` in `package.toml`
- `bump_package` / `bump_all` — updates `versions.toml`

### `PackageManifest` (`magic_pkg/src/manifest.rs`)

Fields: `category`, `vendor`, `name`, `version`, `enabled`, `targets: BTreeSet<PackageTarget>`, `root: PathBuf`

`PackageTarget` enum: `Editor` → directory `"editor"`, `Runtime` → directory `"runtime"`

### `ProjectPackagesManifest` (`magic_pkg/src/manifest.rs`)

Wraps `packages.toml` with `dependencies: BTreeMap<String, String>` (key = `vendor/name`, value = version).

### `utils.rs` helper functions

- `extract_zip_archive`, `copy_dir_all` — file operations
- `uuid_lite()` — nanosecond-based UUID
- `find_workspace_root(start_dir)` — walks up to find `Cargo.toml` with `[workspace]`
- `find_crate_manifest(workspace_root, crate_name)` — locates a crate's `Cargo.toml`
- `update_manifest_dependencies(manifest_path, dependencies)` — rewrites `[dependencies]` section with `package_*` aliases, preserves manual edits outside auto-generated block
- `update_editor_lib_register` / `update_runtime_lib_register` — writes `lib.rs` with `register_all_packages` calling each package's `register`
- `cargo_identifier(input)` — sanitizes string for use as Rust identifier
- `relative_path(from, to)`, `toml_path(path)` — path helpers
- `get_workspace_engine_version(workspace_root)` — reads `[package].version` from root `Cargo.toml`
- Table printers: `print_packages_table`, `print_outdated_table`, `print_releases_table`

### Error Handling

`PackageError` enum: `Io`, `Manifest`, `InvalidManifest`, `DuplicateId`, `AlreadyExists`, `InvalidArgument`, `DownloadError`, `ZipError`.

---

## 5. `magic_runtime` — Runtime Crate

Minimal crate with:
- `pub fn add(left: u64, right: u64) -> u64`
- Unit test `it_works`

Contains `magic_pkg_runtime/` sub-crate (auto-generated, empty `register_all_packages`).

---

## 6. Data Flow Summary

```
User edits .rs / Cargo.toml / package.toml
        ↓
notify watcher detects change (ignores target/, .git/)
        ↓
1.5s debounce → cargo check → cargo build -p magic_editor --lib
        ↓
Success → libloading loads libmagic_editor.so
        ↓
Calls register_all_packages(&mut EditorRegistry)
        ↓
Packages register menu items / event callbacks
        ↓
UI draws top bar with menus + compile status LED
```

Package management flow:
```
magic_pkg init → creates packages/<cat>/<vendor>/<name>/
magic_pkg sync → updates magic_pkg_editor / magic_pkg_runtime Cargo.toml + lib.rs
magic_editor cdylib rebuilds → register_all_packages calls each package's register()
```
