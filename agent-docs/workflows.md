# Workflows

## 1. Hot-Reload Workflow

### Trigger
File system watcher (`notify`) monitors:
- `./packages/**` — recursive
- `./magic_editor/**` — recursive

Valid file types: `.rs`, `Cargo.toml`, `Cargo.lock`, `package.toml`, anything under `/packages/`  
Ignored: `target/`, `.git/`

Note: The project uses TOML format (`package.toml`, `packages.toml`, `versions.toml`), not JSON.

### Debounce
- Change detected → `pending = true`, `last_change = Instant::now()`
- Background thread checks every 200ms
- Compiles only if `!compiling && elapsed >= 1500ms`

### Compile
1. `cargo check -p magic_editor --message-format=short`
2. If check passes: `cargo build -p magic_editor --lib`
3. On build success → `CompilerMessage::CompileSuccess`

### Reload
`reload_and_load_library()`:
1. Clears `menu_registry` and `editor_events`
2. Drops `current_lib`
3. Loads platform-specific library:
   - Linux: `./target/debug/libmagic_editor.so`
   - Windows: `./target/debug/magic_editor.dll`
   - macOS: `./target/debug/libmagic_editor.dylib`
4. Resolves symbol `register_all_packages` via `libloading`
5. Creates `EditorRegistry { menu: &mut menu_registry, events: &mut editor_events }`
6. Calls `register_all_packages(&mut editor_registry)`
7. Stores library in `current_lib` (keeps it alive)

### Manual Reload
`EngineCompilerState::global().trigger_recompile()` sets `request_reload = true`.  
On next UI `update()`, if flag is set, `execute_compile()` is called.

---

## 2. Package Creation Workflow (`magic_pkg init`)

```
magic_pkg init --category editor --package vendor/name --editor --runtime
```

1. Parses `vendor/name` → validates segments (ASCII alphanumeric, `_`, `-`)
2. Creates `packages/<category>/<vendor>/<name>/`
3. Creates `assets/previews_pkg/.gitkeep`
4. Writes `package.toml` with name, vendor, version, category, magic version, channel, status, targets
5. For each target, writes target crate:
   - `<name>/<target>/Cargo.toml` — depends on `magic_core`
   - `<name>/<target>/src/lib.rs` — empty `register()` function
6. Calls `update_versions_file()` → creates/updates `versions.toml`
7. Adds `vendor/name = "0.1.0"` to `packages.toml`
8. Calls `self.sync()` → updates `magic_pkg_editor` / `magic_pkg_runtime` Cargo.toml + lib.rs

---

## 3. Package Sync Workflow (`magic_pkg sync`)

```
magic_pkg sync
```

1. Loads `packages.toml` (project manifest)
2. For each dependency `vendor/name = version`:
   - If package dir doesn't exist under `packages/`, installs it (from URL or repo)
3. Discovers all packages in `packages/`
4. For each discovered package not in manifest, adds it to manifest
5. For each package's dependencies, adds missing ones to manifest
6. Saves manifest if updated
7. Discovers packages again, counts enabled ones
8. For each enabled package:
   - Reads `<name>/<target>/Cargo.toml` to get crate name
   - Builds dependency list: `package_<vendor>_<name> = { package = "<crate_name>", path = "<rel_path>" }`
9. Updates `magic_pkg_editor/Cargo.toml` `[dependencies]` with editor deps
10. Updates `magic_pkg_runtime/Cargo.toml` `[dependencies]` with runtime deps
11. Updates `magic_editor/magic_pkg_editor/src/lib.rs` — writes `register_all_packages` calling each editor package's `register`
12. Updates `magic_runtime/magic_pkg_runtime/src/lib.rs` — writes `register_all_packages` calling each runtime package's `register`
13. Returns `SyncReport`

---

## 4. Package Install Workflow (`magic_pkg install`)

### From local zip
```
magic_pkg install ./package.zip
```
1. Extracts zip to temp dir
2. Finds `package.toml` (root or subdirectory)
3. Parses `RawManifest`
4. Copies contents to `packages/<category>/<vendor>/<name>/`
5. Caches zip to `~/.magic-bevy/packages/<vendor>/<name>/<version>.zip`
6. Adds dependency to `packages.toml`
7. Calls `read_manifest()` → returns `PackageManifest`

### From GitHub repo
```
magic_pkg install vendor/name
magic_pkg install vendor/name#v1.2.3
```
1. If version empty → `resolve_latest_compatible_version()` fetches `versions.toml` from `https://raw.githubusercontent.com/<vendor>/<name>/main/versions.toml` (fallback to `master`)
2. Filters releases by `magic == engine_version` and `channel == "stable"`
3. Downloads `https://github.com/<vendor>/<name>/archive/refs/tags/v<version>.zip`
4. Delegates to `install_from_zip`

---

## 5. Package Update Workflow (`magic_pkg update`)

- Single package: `magic_pkg update vendor/name` → calls `install_repo(vendor, name, "")` (resolves latest)
- All packages: `magic_pkg update` → discovers all, updates each, then `sync()`

---

## 6. Status Toggle (`magic_pkg enable` / `disable`)

```
magic_pkg enable vendor/name
magic_pkg disable vendor/name
```

1. Finds `package.toml` under `packages/*/<vendor>/<name>/`
2. Parses TOML, sets `status = "enabled"` or `"disabled"`, removes conflicting `enabled` key
3. Writes back
4. Calls `sync()`

---

## 7. Version Bump (`magic_pkg bump`)

```
magic_pkg bump vendor/name
magic_pkg bump
```

1. Reads `package.toml` from package root
2. Updates/creates `versions.toml` with:
   - `version` from `package.toml`
   - `magic` from `package.toml.magic` (default `"0.1.0"`)
   - `timestamp` = current Unix epoch seconds
   - `channel` from `package.toml.channel` (default `"stable"`)
3. Sorts releases by timestamp descending
