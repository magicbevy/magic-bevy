# Dev Status - MagicBevy

## Current Phase: Oracle Logs Console

### ✅ Completed
- Centralized logging system: `LogEntry`, `LogLevel`, `LogStore`, `global_store` (OnceLock + Mutex singleton)
- Log macros: `oracle_info!`, `oracle_warn!`, `oracle_error!`, `oracle_success!`, `oracle_debug!`
- `OracleLogsWindow` dockable panel: per-log row with level icon (E/W/I/S/D), timestamp, level tag, short title
- Collapsible details (`v`/`>`) showing full multi-line body in dark frame
- Level filters (all on by default), timestamps toggle, auto-scroll
- Clear button, entry counter, selected counter
- Panic hook in `main.rs` logs crash payload + location to Oracle Logs
- Cargo build errors flow to Oracle Logs via `CompileError` → `oracle_error!`
- `PackageManager::sync()` errors logged to Oracle Logs
- File watcher creation errors logged to Oracle Logs
- `serde_json` serialization errors in layouts logged to Oracle Logs
- Save/Delete layout dialogs anchored to right side of screen
- Removed `TestConsoleWindow` mock window

### 🐛 Bug Fixes
- TestConsoleWindow removed from `mock_packages()`

### 🔄 In Progress
- None

### ⏳ Next Planned
- Package Manager Window
- File Manager Window

### 🚫 Blocked
- None

---

## File Changes (current session)

| File | Change |
|------|--------|
| `magic_core/src/log/mod.rs` | New: LogEntry, LogLevel, LogStore, global_store; exports macros module |
| `magic_core/src/log/store.rs` | New: LogStore with push/entries/clear, `global_store()` singleton |
| `magic_core/src/log/macros.rs` | New: oracle_info!, oracle_warn!, oracle_error!, oracle_success!, oracle_debug! macros |
| `magic_core/src/ui/mod.rs` | Added `oracle_logs` module |
| `magic_core/src/ui/oracle_logs.rs` | New: OracleLogsWindow dockable panel with filters, timestamps, expand/collapse |
| `magic_core/src/lib.rs` | Added `log` module and `OracleLogsWindow` export |
| `magic_core/src/ui/top_bar.rs` | Save/Delete layout dialogs anchored right |
| `src/main.rs` | Added global panic hook logging to Oracle Logs |
| `src/app.rs` | `CompileError` → `oracle_error!`; sync errors → `oracle_error!`; removed TestConsoleWindow |
| `src/hot_reload.rs` | Watcher creation failure → `oracle_error!`; cargo stderr in CompileError |
| `src/layouts.rs` | serde_json unwraps → `unwrap_or_else` + `oracle_error!` |
| `magic_core/Cargo.toml` | Added `chrono` dependency |

---

## Build Status
- `cargo check` — **PASS** (no warnings, no errors)
