use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use magic_core::*;

use crate::app::MagicBevyApp;

pub enum CompilerMessage {
    Indexing(String, usize),
    CompileSuccess,
    RuntimeBuildSuccess,
    CompileError(String),
}

pub fn execute_compile(
    tx: Sender<CompilerMessage>,
    is_compiling: Arc<Mutex<bool>>,
    ctx: eframe::egui::Context,
    build_editor: bool,
    build_runtime: bool,
    run_sync: bool,
) {
    {
        let mut lock = is_compiling.lock().unwrap();
        if *lock {
            return;
        }
        *lock = true;
    }

    let _ = tx.send(CompilerMessage::Indexing(
        "Compiling dynamic module...".to_string(),
        1,
    ));
    ctx.request_repaint();

    let ctx_clone = ctx.clone();
    thread::spawn(move || {
        // Optional package sync
        if run_sync {
            let _ = std::process::Command::new("cargo")
                .args(["run", "-p", "magic_pkg", "--", "sync"])
                .output();
        }

        // Build targets
        let mut args = vec!["build".to_string()];
        if build_editor {
            args.push("-p".to_string());
            args.push("magic_editor".to_string());
            args.push("--lib".to_string());
        }
        if build_runtime {
            args.push("-p".to_string());
            args.push("magic_runtime".to_string());
        }

        // If nothing to build, just finish
        if !build_editor && !build_runtime {
            let _ = tx.send(CompilerMessage::CompileSuccess);
            *is_compiling.lock().unwrap() = false;
            ctx_clone.request_repaint();
            return;
        }

        // cargo build doesn't accept multiple -p with mixed --lib well; run per target instead
        let mut err_msg = String::new();

        if build_editor {
            let out = std::process::Command::new("cargo")
                .args(["build", "-p", "magic_editor", "--lib"])
                .output();
            match out {
                Ok(o) if o.status.success() => {}
                Ok(o) => {
                    err_msg.push_str(&String::from_utf8_lossy(&o.stderr));
                }
                Err(e) => {
                    err_msg.push_str(&e.to_string());
                }
            }
        }

        if build_runtime && err_msg.is_empty() {
            let out = std::process::Command::new("cargo")
                .args(["build", "-p", "magic_runtime", "--lib"])
                .output();
            match out {
                Ok(o) if o.status.success() => {}
                Ok(o) => {
                    err_msg.push_str(&String::from_utf8_lossy(&o.stderr));
                }
                Err(e) => {
                    err_msg.push_str(&e.to_string());
                }
            }
        }

        if !err_msg.is_empty() {
            let _ = tx.send(CompilerMessage::CompileError(format!("Build failed.\n{}", err_msg)));
        } else {
            if build_editor {
                let _ = tx.send(CompilerMessage::CompileSuccess);
            }
            if build_runtime {
                let _ = tx.send(CompilerMessage::RuntimeBuildSuccess);
            }
        }

        *is_compiling.lock().unwrap() = false;
        ctx_clone.request_repaint();
    });
}

pub fn init_file_watcher(
    tx: Sender<CompilerMessage>,
    is_compiling: Arc<Mutex<bool>>,
    ctx: eframe::egui::Context,
) -> notify::RecommendedWatcher {
    let last_change = Arc::new(Mutex::new(Instant::now()));
    #[derive(Clone, Copy, Default)]
    struct PendingTargets { editor: bool, runtime: bool, sync: bool, any: bool }
    let pending = Arc::new(Mutex::new(PendingTargets::default()));

    let tx_clone = tx.clone();
    let is_compiling_clone = Arc::clone(&is_compiling);
    let last_change_clone = Arc::clone(&last_change);
    let pending_clone = Arc::clone(&pending);
    let ctx_thread = ctx.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(200));

            let mut pending_guard = pending_clone.lock().unwrap();
            if pending_guard.any {
                let compiling = *is_compiling_clone.lock().unwrap();
                let last_event_time = *last_change_clone.lock().unwrap();

                if !compiling && last_event_time.elapsed() >= Duration::from_millis(1000) {
                    let be = pending_guard.editor;
                    let br = pending_guard.runtime;
                    let rs = pending_guard.sync;
                    *pending_guard = PendingTargets::default();
                    drop(pending_guard);
                    MagicBevyApp::execute_compile_static(
                        tx_clone.clone(),
                        Arc::clone(&is_compiling_clone),
                        ctx_thread.clone(),
                        be,
                        br,
                        rs,
                    );
                }
            }
        }
    });

    let last_change_watcher = Arc::clone(&last_change);
    let pending_watcher = Arc::clone(&pending);
    let tx_watcher = tx.clone();
    let ctx_watcher = ctx.clone();

    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if *EngineCompilerState::global().lock_mode.lock().unwrap()
            == CompilerLockMode::EditorLock
        {
            return;
        }

        if let Ok(event) = res {
            if event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove() {
                let mut build_editor = false;
                let mut build_runtime = false;
                let mut run_sync = false;
                let mut any_valid = false;

                for pp in &event.paths {
                    let path_str = pp.to_string_lossy();

                    if path_str.contains("/target/")
                        || path_str.contains("\\target\\")
                        || path_str.contains("/.git/")
                        || path_str.contains("\\.git\\")
                    {
                        continue;
                    }

                    let is_valid = path_str.ends_with(".rs")
                        || path_str.ends_with("package.json")
                        || path_str.ends_with("Cargo.toml")
                        || path_str.ends_with("Cargo.lock")
                        || path_str.ends_with("package.toml")
                        || path_str.contains("/packages/")
                        || path_str.contains("\\packages\\")
                        || pp.components().any(|c| c.as_os_str() == "packages")
                        || path_str.contains("magic_editor")
                        || path_str.contains("magic_runtime")
                        || path_str.contains("magic_api_editor")
                        || path_str.contains("magic_api_runtime")
                        || path_str.contains("magic_log")
                        || path_str.contains("magic_core");

                    if !is_valid {
                        continue;
                    }
                    any_valid = true;

                    if path_str.ends_with("package.toml")
                        || path_str.ends_with("Cargo.toml")
                        || path_str.ends_with("Cargo.lock")
                        || path_str.ends_with("package.json")
                    {
                        run_sync = true;
                    }

                    if path_str.contains("magic_editor")
                        || path_str.contains("magic_core")
                        || path_str.contains("magic_api_editor")
                        || path_str.contains("magic_log")
                        || path_str.contains("/editor/")
                        || path_str.contains("\\editor\\")
                    {
                        build_editor = true;
                    }

                    if path_str.contains("magic_runtime")
                        || path_str.contains("magic_api_runtime")
                        || path_str.contains("magic_log")
                        || path_str.contains("/runtime/")
                        || path_str.contains("\\runtime\\")
                    {
                        build_runtime = true;
                    }

                    // .rs files under packages default to editor unless under runtime/
                    if path_str.ends_with(".rs")
                        && (path_str.contains("/packages/") || path_str.contains("\\packages\\"))
                        && !path_str.contains("/runtime/") && !path_str.contains("\\runtime\\")
                    {
                        build_editor = true;
                    }
                }

                if any_valid {
                    *last_change_watcher.lock().unwrap() = Instant::now();
                    let mut g = pending_watcher.lock().unwrap();
                    g.editor |= build_editor;
                    g.runtime |= build_runtime;
                    g.sync |= run_sync;
                    g.any = true;
                    drop(g);

                    let _ = tx_watcher.send(CompilerMessage::Indexing(
                        "Indexing package changes...".to_string(),
                        1,
                    ));
                    ctx_watcher.request_repaint();
                }
            }
        }
    })
    .unwrap_or_else(|e| {
        magic_log::oracle_error!("HotReload", "Failed to create file watcher: {}", e);
        panic!("file watcher creation failed: {}", e);
    });

    let packages_path = Path::new("./packages");
    if packages_path.exists() {
        if let Ok(absolute_packages) = packages_path.canonicalize() {
            let _ = watcher.watch(&absolute_packages, RecursiveMode::Recursive);
        }
    }

    for watch_path in [
        "./magic_editor",
        "./magic_runtime",
        "./magic_api_editor",
        "./magic_api_runtime",
        "./magic_log",
        "./magic_core",
    ] {
        let pth = Path::new(watch_path);
        if pth.exists() {
            if let Ok(abs) = pth.canonicalize() {
                let _ = watcher.watch(&abs, RecursiveMode::Recursive);
            }
        }
    }

    watcher
}

impl MagicBevyApp {
    fn execute_compile_static(
        tx: Sender<CompilerMessage>,
        is_compiling: Arc<Mutex<bool>>,
        ctx: eframe::egui::Context,
        build_editor: bool,
        build_runtime: bool,
        run_sync: bool,
    ) {
        crate::hot_reload::execute_compile(tx, is_compiling, ctx, build_editor, build_runtime, run_sync);
    }
}
