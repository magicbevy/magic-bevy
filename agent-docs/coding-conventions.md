# Coding Conventions

## General

- **Language:** Rust, edition 2024
- **Formatting:** Standard rustfmt (no custom config found)
- **Comments:** Minimal. Do not add comments unless asked (per project guidelines)
- **Error handling:** Use `Result` with custom error enums (`PackageError` in `magic_pkg`)
- **Logging:** Use `mgprint!`, `mgsuccess!`, `mgwarn!`, `mgerror!` macros from `magic_core::logger`
- **String tags:** Use `const TAG: &str = "ModuleName";` at top of files that log

## Naming

- Crates: snake_case (e.g., `magic_core`, `magic_pkg`)
- Modules: snake_case
- Structs/Enums/Traits: PascalCase (e.g., `MagicBevyApp`, `CompilerStatus`, `EditorSystem`)
- Functions/Methods: snake_case
- Constants: UPPER_SNAKE or `Tag` style (e.g., `const TAG: &str = "PackageManager";`)
- Package ID: `vendor/name` format
- Cargo alias: `package_<vendor>_<name>` (sanitized via `cargo_identifier()`)

## File Organization

```
magic_core/src/
  lib.rs          # module declarations + re-exports
  compiler.rs     # global compiler state
  logger.rs       # colored macros
  api/            # EditorRegistry, MenuRegistry, events, runtime
  status/         # CompileStatus, CompilerStateManager
  ui/             # top_bar, indicator
  loader/         # editor_lib, package_reloader
  package_manager/ # empty

magic_editor/src/
  lib.rs          # cdylib export: register_all_packages
  systems/        # EditorSystem trait + SystemManager
  profile/        # placeholder

magic_pkg/src/
  lib.rs          # module declarations
  main.rs         # CLI entry point (clap)
  error.rs        # PackageError
  manifest.rs     # PackageTarget, PackageManifest, etc.
  manager.rs      # PackageManager — all operations
  utils.rs        # helpers, manifest rewriting, table printers
```

## Concurrency

- Use `std::sync::mpsc` channels for thread-to-UI communication
- Use `Arc<Mutex<T>>` for shared mutable state (e.g., `is_compiling`)
- File watcher callbacks run in `notify` thread; repaint via `ctx.request_repaint()`
- Compilation runs in spawned `std::thread`

## Unsafe

- `libloading` requires `unsafe` blocks for `Library::new()` and `lib.get()`
- `#[no_mangle] extern "Rust" fn` used for exported cdylib symbols

## TOML Handling

- `package.toml` per package (name, vendor, version, category, magic, channel, status, targets, dependencies)
- `packages.toml` at project root (dependencies map: `vendor/name = "version"`)
- `versions.toml` per package (release history with version, magic, timestamp, channel)

## When Adding a New Package

1. Create under `packages/<category>/<vendor>/<name>/`
2. Add `package.toml`
3. Create `editor/` and/or `runtime/` sub-crates with `Cargo.toml` + `src/lib.rs`
4. Run `magic_pkg sync` to wire dependencies into `magic_pkg_editor` / `magic_pkg_runtime`
5. The `register_all_packages` function in each sub-crate's `lib.rs` will be auto-generated

## When Modifying Hot-Reload Logic

- Edit `src/main.rs`: `execute_compile()`, `reload_and_load_library()`, `init_file_watcher()`
- Library paths are platform-specific (`cfg(target_os = ...)`)
- The loaded library must export `register_all_packages` with signature `extern "Rust" fn(&mut EditorRegistry)`
