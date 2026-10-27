use eframe::egui;
use std::path::Path;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};

use magic_core::*;
use magic_pkg::PackageManager;

use crate::hot_reload::CompilerMessage;
use crate::windows::WindowManager;
pub struct MagicBevyApp {
    menu_registry: MenuRegistry,
    editor_events: EditorEventRegistry,
    windows: WindowRegistry,
    window_manager: WindowManager,
    compiler_manager: CompilerStateManager,
    tx: Sender<CompilerMessage>,
    rx: Receiver<CompilerMessage>,
    is_compiling: Arc<Mutex<bool>>,
    _watcher: notify::RecommendedWatcher,
    current_lib: Option<libloading::Library>,
    save_name: String,
    show_save_dialog: bool,
    delete_confirm: Option<(String, bool)>,
    current_layout_name: Option<String>,
}

impl MagicBevyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        let (tx, rx) = channel();
        let is_compiling = Arc::new(Mutex::new(false));

        let watcher = crate::hot_reload::init_file_watcher(
            tx.clone(),
            Arc::clone(&is_compiling),
            cc.egui_ctx.clone(),
        );

        let mut app = Self {
            menu_registry: MenuRegistry::new(),
            editor_events: EditorEventRegistry::new(),
            windows: WindowRegistry::new(),
            window_manager: WindowManager::new(),
            compiler_manager: CompilerStateManager::new(),
            tx,
            rx,
            is_compiling,
            _watcher: watcher,
            current_lib: None,
            save_name: String::new(),
            show_save_dialog: false,
            delete_confirm: None,
            current_layout_name: None,
        };

        app.compiler_manager.set_success();

        app.reload_and_load_library();

        match PackageManager::new(".").sync() {
            Ok(report) => {
                mgsuccess!("PackageSync", "Workspace synchronized: {} discovered, {} enabled.", report.discovered, report.enabled);
            }
            Err(err) => {
                mgwarn!("PackageSync", "Sync failed: {}", err);
            }
        }

                crate::hot_reload::execute_compile(
            app.tx.clone(),
            Arc::clone(&app.is_compiling),
            cc.egui_ctx.clone(),
        );

        EngineCompilerState::global().unlock_watcher();

        // Load temp layout if exists, otherwise load active layout
        let temp_loaded = app.window_manager.load_temp_layout(&app.windows);
        if temp_loaded {
            app.current_layout_name = None;
        } else {
            let (active_name, is_global, is_project) = {
                let name = app.window_manager.get_active_layout().map(|s| s.to_string());
                match name {
                    Some(n) => {
                        let ig = app.window_manager.layout_manager.is_global_layout(&n);
                        let ip = app.window_manager.layout_manager.is_project_layout(&n);
                        (Some(n), ig, ip)
                    }
                    None => (None, false, false),
                }
            };
            if let Some(n) = active_name {
                app.current_layout_name = Some(n.clone());
                if is_global || is_project {
                    app.window_manager.load_layout_named(&app.windows, &n, is_global);
                }
            } else {
                app.current_layout_name = None;
            }
        }
        app
    }

    fn reload_and_load_library(&mut self) {
        mgprint!("HotReload", "Clearing resources and loading new library...");

        self.menu_registry.clear();
        self.editor_events.clear();
        self.windows.descriptors.clear();
        self.windows.factories.clear();
        self.current_lib = None;

        #[cfg(target_os = "linux")]
        let lib_path = "./target/debug/libmagic_editor.so";
        #[cfg(target_os = "windows")]
        let lib_path = "./target/debug/magic_editor.dll";
        #[cfg(target_os = "macos")]
        let lib_path = "./target/debug/libmagic_editor.dylib";

        if Path::new(lib_path).exists() {
            unsafe {
                if let Ok(lib) = libloading::Library::new(lib_path) {

                    type RegisterFn = unsafe extern "Rust" fn(&mut EditorRegistry);

                    if let Ok(register_all_packages) = lib.get::<RegisterFn>(b"register_all_packages\0") {
                        let mut editor_registry = EditorRegistry {
                            menu: &mut self.menu_registry,
                            events: &mut self.editor_events,
                            windows: &mut self.windows,
                            open_window_tx: self.window_manager.tx.clone(),
                        };

                        register_all_packages(&mut editor_registry);
                        mgsuccess!("HotReload", "Dynamic packages registered successfully.");

                        // Add Window menu items for each registered window type
                        for descriptor in &self.windows.descriptors {
                            let tx = self.window_manager.tx.clone();
                            let window_id = descriptor.id.clone();
                            self.menu_registry.register_action(
                                &["window"],
                                &format!("Open {}", descriptor.label),
                                Box::new(move || {
                                    let _ = tx.send(window_id.clone());
                                }),
                            );
                        }

                        // Add layout management items
                        let project_layouts = self.window_manager.list_project_layouts();
                        let global_layouts = self.window_manager.list_global_layouts();
                        for layout_name in project_layouts.iter().chain(global_layouts.iter()) {
                            self.menu_registry.register_action(
                                &["window", "Layouts"],
                                &format!("Load {}", layout_name),
                                Box::new(move || {
                                    // Handled in update via pending load
                                }),
                            );
                        }
                    } else {
                        mgwarn!(
                            "HotReload",
                            "Failed to resolve 'register_all_packages' symbol from binary."
                        );
                    }

                    self.current_lib = Some(lib);
                } else {
                    mgerror!(
                        "HotReload",
                        "Error occurred while loading magic_editor dynamic library."
                    );
                }
            }
        } else {
            mgwarn!("HotReload", "Library file not found at: {}", lib_path);
        }

        self.mock_packages();
    }

    fn mock_packages(&mut self) {
        self.menu_registry
            .register_action(&["file"], "Save Scene", Box::new(|| println!("Scene Saved!")));
        self.menu_registry
            .register_action(&["file"], "Exit", Box::new(|| std::process::exit(0)));

        // Test window to verify window system works
        self.windows.register(
            WindowDescriptor {
                id: "test/console".to_string(),
                label: "Test Console".to_string(),
                default_width: 400.0,
                default_height: 300.0,
                dockable: true,
            },
            WindowFactory {
                create: Box::new(|| {
                    Box::new(TestConsoleWindow::new()) as Box<dyn EditorWindow>
                }),
            },
        );

        // Add menu item for test console
        let tx = self.window_manager.tx.clone();
        self.menu_registry.register_action(
            &["window"],
            "Open Test Console",
            Box::new(move || {
                let _ = tx.send("test/console".to_string());
            }),
        );
    }
}

struct TestConsoleWindow {
    messages: Vec<String>,
}

impl TestConsoleWindow {
    fn new() -> Self {
        Self {
            messages: vec![
                "[INFO] MagicBevy started".to_string(),
                "[INFO] Window system initialized".to_string(),
                "[INFO] Test console window ready".to_string(),
            ],
        }
    }
}

impl EditorWindow for TestConsoleWindow {
    fn id(&self) -> WindowId {
        "test/console".to_string()
    }

    fn title(&self) -> &str {
        "Test Console"
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.heading("Test Console");
        ui.separator();
        for msg in &self.messages {
            ui.label(msg);
        }
        ui.separator();
        if ui.button("Add Log Entry").clicked() {
            self.messages.push("[INFO] New log entry".to_string());
        }
    }
}

impl eframe::App for MagicBevyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let compiler_state = EngineCompilerState::global();
        if let Ok(mut req) = compiler_state.request_reload.lock() {
            if *req {
                *req = false;
                let _ = self.tx.send(CompilerMessage::Indexing(
                    "Compiling dynamic module...".to_string(),
                    1,
                ));
                crate::hot_reload::execute_compile(
                    self.tx.clone(),
                    Arc::clone(&self.is_compiling),
                    ui.ctx().clone(),
                );
            }
        }

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                CompilerMessage::Indexing(file, count) => {
                    self.compiler_manager.set_compiling(&file, count);
                }
                CompilerMessage::CompileSuccess => {
                    self.compiler_manager.set_success();
                    self.reload_and_load_library();
                }
                CompilerMessage::CompileError(_err) => {
                    self.compiler_manager.set_error("Error");
                }
            }
        }

        // Process window open requests
        while let Ok(window_id) = self.window_manager.rx.try_recv() {
            self.window_manager.open_window(&self.windows, &window_id);
        }

        // --- Top bar with menus ---
        let ctx = (*ui.ctx()).clone();
        let project_layouts = self.window_manager.list_project_layouts();
        let global_layouts = self.window_manager.list_global_layouts();
        let layout_action = draw_top_bar(
            &ctx,
            ui,
            &self.menu_registry,
            &self.compiler_manager.status,
            &project_layouts,
            &global_layouts,
            &mut self.current_layout_name,
            &mut self.save_name,
            &mut self.show_save_dialog,
            &mut self.delete_confirm,
        );

        match layout_action {
            LayoutAction::Save { name, is_global } => {
                if name.is_empty() {
                    let name = format!(
                        "layout_{}",
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                    );
                    self.window_manager.save_layout_named(&name, is_global);
                } else {
                    self.window_manager.save_layout_named(&name, is_global);
                }
                self.window_manager.set_active_layout(&name);
                self.current_layout_name = Some(name.clone());
            }
            LayoutAction::Load { name, is_global } => {
                self.window_manager.load_layout_named(&self.windows, &name, is_global);
                self.window_manager.set_active_layout(&name);
                self.current_layout_name = Some(name.clone());
            }
            LayoutAction::Delete { name, is_global } => {
                self.window_manager.delete_layout(&name, is_global);
                if self.current_layout_name.as_ref() == Some(&name.to_string()) {
                    self.current_layout_name = None;
                }
            }
            LayoutAction::None => {}
        }

        // --- Render all open windows ---
        self.window_manager.render_windows(ui, &mut self.windows);
    }
}
