use notify::{Event, RecursiveMode, Watcher};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use magic_core::*;

use crate::app::MagicBevyApp;

pub enum CompilerMessage {
    Indexing(String, usize),
    CompileSuccess,
    CompileError(String),
}

pub fn execute_compile(
    tx: Sender<CompilerMessage>,
    is_compiling: Arc<Mutex<bool>>,
    ctx: eframe::egui::Context,
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
        let check_output = Command::new("cargo")
            .args(["check", "-p", "magic_editor", "--message-format=short"])
            .output();

        if let Ok(output) = check_output {
            if output.status.success() {
                let build_output = Command::new("cargo")
                    .args(["build", "-p", "magic_editor", "--lib"])
                    .output();

                if build_output.is_ok() && build_output.as_ref().unwrap().status.success() {
                    let _ = tx.send(CompilerMessage::CompileSuccess);
                } else {
                    let err = if let Ok(out) = build_output {
                        String::from_utf8_lossy(&out.stderr).to_string()
                    } else {
                        "Build failed: unable to capture cargo output.".to_string()
                    };
                    let _ = tx.send(CompilerMessage::CompileError(
                        format!("Build failed.\n{}", err),
                    ));
                }
            } else {
                let err = String::from_utf8_lossy(&output.stderr).to_string();
                let _ = tx.send(CompilerMessage::CompileError(
                    format!("Cargo check failed.\n{}", err),
                ));
            }
        } else {
            let _ = tx.send(CompilerMessage::CompileError(
                "Failed to invoke cargo check.".to_string(),
            ));
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
    let pending = Arc::new(Mutex::new(false));

    let tx_clone = tx.clone();
    let is_compiling_clone = Arc::clone(&is_compiling);
    let last_change_clone = Arc::clone(&last_change);
    let pending_clone = Arc::clone(&pending);
    let ctx_thread = ctx.clone();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(200));

            let mut is_pending = pending_clone.lock().unwrap();
            if *is_pending {
                let compiling = *is_compiling_clone.lock().unwrap();
                let last_event_time = *last_change_clone.lock().unwrap();

                if !compiling && last_event_time.elapsed() >= Duration::from_millis(1500) {
                    *is_pending = false;
                    MagicBevyApp::execute_compile_static(
                        tx_clone.clone(),
                        Arc::clone(&is_compiling_clone),
                        ctx_thread.clone(),
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
                let is_valid_change = event.paths.iter().any(|p| {
                    let path_str = p.to_string_lossy();

                    if path_str.contains("/target/")
                        || path_str.contains("\\target\\")
                        || path_str.contains("/.git/")
                        || path_str.contains("\\.git\\")
                    {
                        return false;
                    }

                    path_str.ends_with(".rs")
                        || path_str.ends_with("package.json")
                        || path_str.ends_with("Cargo.toml")
                        || path_str.ends_with("Cargo.lock")
                        || path_str.contains("/packages/")
                        || path_str.contains("\\packages\\")
                        || p.components().any(|c| c.as_os_str() == "packages")
                });

                if is_valid_change {
                    *last_change_watcher.lock().unwrap() = Instant::now();
                    *pending_watcher.lock().unwrap() = true;

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
        magic_core::oracle_error!("HotReload", "Failed to create file watcher: {}", e);
        panic!("file watcher creation failed: {}", e);
    });

    let packages_path = Path::new("./packages");
    if packages_path.exists() {
        if let Ok(absolute_packages) = packages_path.canonicalize() {
            let _ = watcher.watch(&absolute_packages, RecursiveMode::Recursive);
        }
    }

    let editor_src_path = Path::new("./magic_editor");
    if editor_src_path.exists() {
        if let Ok(absolute_editor) = editor_src_path.canonicalize() {
            let _ = watcher.watch(&absolute_editor, RecursiveMode::Recursive);
        }
    }

    watcher
}

impl MagicBevyApp {
    fn execute_compile_static(
        tx: Sender<CompilerMessage>,
        is_compiling: Arc<Mutex<bool>>,
        ctx: eframe::egui::Context,
    ) {
        crate::hot_reload::execute_compile(tx, is_compiling, ctx);
    }
}
