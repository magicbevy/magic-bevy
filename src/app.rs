use eframe::egui::{self, Color32, CornerRadius, Margin, Stroke, Vec2};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};

use magic_core::*;
use magic_pkg::PackageManager;
use magic_pkg::PackageTarget;

use crate::hot_reload::CompilerMessage;
use crate::windows::WindowManager;




pub struct EngineTheme;

impl EngineTheme {
    pub const BG_DEEP: Color32 = Color32::from_rgb(12, 13, 18);
    pub const BG_DARK: Color32 = Color32::from_rgb(16, 17, 22);
    pub const BG_PANEL: Color32 = Color32::from_rgb(22, 24, 31);
    pub const BG_BAR: Color32 = Color32::from_rgb(28, 31, 40);

    pub const ITEM_BG: Color32 = Color32::from_rgb(30, 33, 43);
    pub const ITEM_HOVER: Color32 = Color32::from_rgb(38, 43, 56);
    pub const ITEM_SELECTED: Color32 = Color32::from_rgb(42, 48, 75);

    pub const BORDER: Color32 = Color32::from_rgb(45, 50, 65);
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgb(110, 86, 240);

    pub const ACCENT: Color32 = Color32::from_rgb(110, 86, 240);
    pub const ACCENT_CYAN: Color32 = Color32::from_rgb(56, 189, 248);

    pub const WARNING: Color32 = Color32::from_rgb(245, 185, 66);
    pub const DANGER: Color32 = Color32::from_rgb(239, 68, 68);

    pub const TEXT_MAIN: Color32 = Color32::from_rgb(238, 240, 246);
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(145, 152, 170);


    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
        let mut visuals = egui::Visuals::dark();

        visuals.dark_mode = true;


        visuals.panel_fill = Self::BG_DARK;
        visuals.window_fill = Self::BG_PANEL;
        visuals.extreme_bg_color = Self::BG_DEEP;
        visuals.faint_bg_color = Color32::from_rgb(20, 22, 29);
        visuals.code_bg_color = Self::BG_DEEP;


        visuals.hyperlink_color = Self::ACCENT_CYAN;
        visuals.warn_fg_color = Self::WARNING;
        visuals.error_fg_color = Self::DANGER;
        visuals.selection.bg_fill = Self::ITEM_SELECTED;
        visuals.selection.stroke = Stroke::new(1.0, Self::ACCENT_CYAN);


        visuals.window_corner_radius = CornerRadius::same(8);
        visuals.window_stroke = Stroke::new(1.0, Self::BORDER);
        visuals.window_highlight_topmost = true;
        visuals.menu_corner_radius = CornerRadius::same(6);

        visuals.window_shadow = egui::epaint::Shadow {
            offset: [0, 8],
            blur: 20,
            spread: 2,
            color: Color32::from_black_alpha(140),
        };
        visuals.popup_shadow = egui::epaint::Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(120),
        };


        visuals.widgets.noninteractive.bg_fill = Self::BG_PANEL;
        visuals.widgets.noninteractive.weak_bg_fill = Self::BG_PANEL;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Self::BORDER);
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Self::TEXT_MAIN);
        visuals.widgets.noninteractive.corner_radius = CornerRadius::same(6);

        visuals.widgets.inactive.bg_fill = Self::ITEM_BG;
        visuals.widgets.inactive.weak_bg_fill = Self::ITEM_BG;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER);
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_MAIN);
        visuals.widgets.inactive.corner_radius = CornerRadius::same(5);
        visuals.widgets.inactive.expansion = 0.0;

        visuals.widgets.hovered.bg_fill = Self::ITEM_HOVER;
        visuals.widgets.hovered.weak_bg_fill = Self::ITEM_HOVER;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::BORDER_ACTIVE);
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
        visuals.widgets.hovered.corner_radius = CornerRadius::same(5);
        visuals.widgets.hovered.expansion = 1.0;

        visuals.widgets.active.bg_fill = Self::ITEM_SELECTED;
        visuals.widgets.active.weak_bg_fill = Self::ACCENT;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, Self::ACCENT_CYAN);
        visuals.widgets.active.fg_stroke = Stroke::new(2.0, Color32::WHITE);
        visuals.widgets.active.corner_radius = CornerRadius::same(5);
        visuals.widgets.active.expansion = 1.0;

        visuals.widgets.open.bg_fill = Self::BG_BAR;
        visuals.widgets.open.weak_bg_fill = Self::BG_BAR;
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, Self::BORDER_ACTIVE);
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, Self::ACCENT_CYAN);
        visuals.widgets.open.corner_radius = CornerRadius::same(5);
        visuals.widgets.open.expansion = 0.0;


        style.spacing.item_spacing = Vec2::new(8.0, 6.0);
        style.spacing.window_margin = Margin::same(10);
        style.spacing.menu_margin = Margin::same(6);
        style.spacing.button_padding = Vec2::new(10.0, 5.0);
        style.spacing.indent = 18.0;
        style.spacing.interact_size = Vec2::new(40.0, 22.0);
        style.spacing.scroll.bar_width = 8.0;

        style.visuals = visuals;
        ctx.set_style_of(egui::Theme::Dark, style.clone());
        ctx.set_style_of(egui::Theme::Light, style);
    }
}

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

        EngineTheme::apply(&cc.egui_ctx);

        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

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
                mgsuccess!(
                    "PackageSync",
                    "Workspace synchronized: {} discovered, {} enabled.",
                    report.discovered,
                    report.enabled
                );
            }
            Err(err) => {
                mgwarn!("PackageSync", "Sync failed: {}", err);
                magic_core::oracle_error!("PackageSync", "Sync failed: {}", err);
            }
        }

        crate::hot_reload::execute_compile(
            app.tx.clone(),
            Arc::clone(&app.is_compiling),
            cc.egui_ctx.clone(),
        );

        EngineCompilerState::global().unlock_watcher();


        let temp_loaded = app.window_manager.load_temp_layout(&app.windows);

        if let Ok(content) = std::fs::read_to_string("./.magic/grimoire_state.json") {
            if let Ok(states) = serde_json::from_str::<Vec<String>>(&content) {
                let grimoire_ids: Vec<u64> = app.window_manager.window_instances.iter_ids().collect::<Vec<_>>()
                    .into_iter()
                    .filter(|id| app.window_manager.window_instances.get_window_id(*id).map(|w| w == "grimoire").unwrap_or(false))
                    .collect();
                for (idx, id) in grimoire_ids.iter().enumerate() {
                    if let Some(state) = states.get(idx) {
                        if let Some(instance) = app.window_manager.window_instances.get_mut(*id) {
                            instance.window.restore_state(state);
                        }
                    }
                }
            }
            let _ = std::fs::remove_file("./.magic/grimoire_state.json");
        }

        if temp_loaded {
            app.current_layout_name = None;
        } else {
            let (active_name, is_global, is_project) = {
                let name = app
                    .window_manager
                    .get_active_layout()
                    .map(|s| s.to_string());
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
                    app.window_manager
                        .load_layout_named(&app.windows, &n, is_global);
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

                    if let Ok(register_all_packages) =
                        lib.get::<RegisterFn>(b"register_all_packages\0")
                    {
                        let mut asset_guard = magic_core::global_asset_registry().lock().unwrap();
                        asset_guard.clear_types();

                        let mut editor_registry = EditorRegistry {
                            menu: &mut self.menu_registry,
                            events: &mut self.editor_events,
                            windows: &mut self.windows,
                            assets: &mut *asset_guard,
                            open_window_tx: self.window_manager.tx.clone(),
                        };

                        register_all_packages(&mut editor_registry);
                        mgsuccess!("HotReload", "Dynamic packages registered successfully.");


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


                        let project_layouts = self.window_manager.list_project_layouts();
                        let global_layouts = self.window_manager.list_global_layouts();
                        for layout_name in project_layouts.iter().chain(global_layouts.iter()) {
                            self.menu_registry.register_action(
                                &["window", "Layouts"],
                                &format!("Load {}", layout_name),
                                Box::new(move || {

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
        self.menu_registry.register_action(
            &["file"],
            "Save Scene",
            Box::new(|| println!("Scene Saved!")),
        );
        self.menu_registry
            .register_action(&["file"], "Exit", Box::new(|| std::process::exit(0)));


        self.windows.register(
            WindowDescriptor {
                id: "oracle_logs".to_string(),
                label: "Oracle Logs".to_string(),
                default_width: 600.0,
                default_height: 400.0,
                dockable: true,
            },
            WindowFactory {
                create: Box::new(|| {
                    Box::new(OracleLogsWindow::new()) as Box<dyn EditorWindow>
                }),
            },
        );


        self.windows.register(
            WindowDescriptor {
                id: "grimoire".to_string(),
                label: "Grimoire".to_string(),
                default_width: 900.0,
                default_height: 600.0,
                dockable: true,
            },
            WindowFactory {
                create: Box::new(|| {
                    let mut grimoire = GrimoireWindow::new();

                    let discover_cb = move || {
                        let pm = PackageManager::new(".");
                        let manifests = pm.discover().unwrap_or_default();
                        manifests
                            .into_iter()
                            .map(|m| {
                                let has_banner = m.root.join("assets/previews_pkg").exists()
                                    && m.root
                                        .join("assets/previews_pkg")
                                        .read_dir()
                                        .map(|d| d.count() > 0)
                                        .unwrap_or(false);
                                PackageInfo {
                                    id: m.id(),
                                    category: m.category,
                                    vendor: m.vendor,
                                    name: m.name,
                                    version: m.version,
                                    enabled: m.enabled,
                                    targets: m
                                        .targets
                                        .iter()
                                        .map(|t| match t {
                                            PackageTarget::Editor => "Editor".to_string(),
                                            PackageTarget::Runtime => "Runtime".to_string(),
                                        })
                                        .collect(),
                                    path: m.root,
                                    author: String::new(),
                                    description: String::new(),
                                    has_banner,
                                }
                            })
                            .collect()
                    };

                    let install_zip_cb = move |path: String| {
                        let pm = PackageManager::new(".");
                        match pm.install_from_zip(std::path::Path::new(&path)) {
                            Ok(m) => Ok(format!("Installed {} v{} from ZIP", m.id(), m.version)),
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    let install_url_cb = move |url: String| {
                        let pm = PackageManager::new(".");
                        match pm.install_from_url(&url) {
                            Ok(m) => Ok(format!("Installed {} v{} from URL", m.id(), m.version)),
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    let remove_cb = move |id: String| {
                        let parts: Vec<&str> = id.splitn(2, '/').collect();
                        if parts.len() != 2 {
                            return Err(format!("Invalid package ID: {}", id));
                        }
                        let pm = PackageManager::new(".");
                        match pm.remove_package(parts[0], parts[1]) {
                            Ok(_) => Ok(format!("Package {} removed", id)),
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    let set_status_cb = move |id: String, enabled: bool| {
                        let parts: Vec<&str> = id.splitn(2, '/').collect();
                        if parts.len() != 2 {
                            return Err(format!("Invalid package ID: {}", id));
                        }
                        let pm = PackageManager::new(".");
                        match pm.set_package_status(parts[0], parts[1], enabled) {
                            Ok(_) => {
                                let status = if enabled { "enabled" } else { "disabled" };
                                Ok(format!("Package {} {}", id, status))
                            }
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    let bump_cb = move |id: String| {
                        let parts: Vec<&str> = id.splitn(2, '/').collect();
                        if parts.len() != 2 {
                            return Err(format!("Invalid package ID: {}", id));
                        }
                        let pm = PackageManager::new(".");
                        match pm.bump_package(parts[0], parts[1]) {
                            Ok(_) => Ok(format!("Package {} bumped", id)),
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    let sync_cb = move || {
                        let pm = PackageManager::new(".");
                        match pm.sync() {
                            Ok(r) => Ok(format!(
                                "Sync complete: {} discovered, {} enabled",
                                r.discovered, r.enabled
                            )),
                            Err(e) => Err(format!("Sync failed: {}", e)),
                        }
                    };

                    let versions_cb = move |id: String| {
                        let parts: Vec<&str> = id.splitn(2, '/').collect();
                        if parts.len() != 2 {
                            return Err(format!("Invalid package ID: {}", id));
                        }
                        let pm = PackageManager::new(".");
                        match pm.get_package_releases(parts[0], parts[1]) {
                            Ok(rf) => Ok(rf.releases.into_iter().map(|r| r.version).collect()),
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    let install_version_cb = move |id: String, version: String| {
                        let parts: Vec<&str> = id.splitn(2, '/').collect();
                        if parts.len() != 2 {
                            return Err(format!("Invalid package ID: {}", id));
                        }
                        let pm = PackageManager::new(".");
                        match pm.install_repo(parts[0], parts[1], &version) {
                            Ok(m) => Ok(format!("Installed {} v{}", m.id(), m.version)),
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    let update_all_cb = move || {
                        let pm = PackageManager::new(".");
                        if let Ok(packages) = pm.discover() {
                            for pkg in packages {
                                let _ = pm.install_repo(&pkg.vendor, &pkg.name, "");
                            }
                        }
                        match pm.sync() {
                            Ok(r) => Ok(format!(
                                "Update complete: {} discovered, {} enabled",
                                r.discovered, r.enabled
                            )),
                            Err(e) => Err(format!("Update failed: {}", e)),
                        }
                    };

                    let init_cb = move |category: String,
                                        vendor: String,
                                        name: String,
                                        editor: bool,
                                        runtime: bool| {
                        let pm = PackageManager::new(".");
                        let mut targets = Vec::new();
                        if editor {
                            targets.push(PackageTarget::Editor);
                        }
                        if runtime {
                            targets.push(PackageTarget::Runtime);
                        }
                        if targets.is_empty() {
                            targets.push(PackageTarget::Editor);
                        }
                        match pm.init_package(&category, &vendor, &name, targets) {
                            Ok((m, _)) => Ok(format!("Package {} created", m.id())),
                            Err(e) => Err(format!("Failed: {}", e)),
                        }
                    };

                    grimoire = grimoire
                        .with_discover(discover_cb)
                        .with_install_zip(install_zip_cb)
                        .with_install_url(install_url_cb)
                        .with_remove(remove_cb)
                        .with_set_status(set_status_cb)
                        .with_bump(bump_cb)
                        .with_sync(sync_cb)
                        .with_update_all(update_all_cb)
                        .with_versions(versions_cb)
                        .with_install_version(install_version_cb)
                        .with_init(init_cb);

                    Box::new(grimoire) as Box<dyn EditorWindow>
                }),
            },
        );


        self.windows.register(
            WindowDescriptor {
                id: "inspector".to_string(),
                label: "Inspector".to_string(),
                default_width: 360.0,
                default_height: 500.0,
                dockable: true,
            },
            WindowFactory {
                create: Box::new(|| {
                    Box::new(InspectorWindow::new()) as Box<dyn EditorWindow>
                }),
            },
        );


        if let Ok(mut reg) = magic_core::global_asset_registry().lock() {
            reg.register(
                magic_core::AssetTypeDescriptor::new("rs", "Rust Script", egui_phosphor::regular::FILE_RS)
                    .category("Script")
                    .color(eframe::egui::Color32::from_rgb(249, 115, 22))
                    .folders(["editor", "runtime"])
                    .subfolders(["src"])
                    .default_name("new_script")
                    .on_create(|dir, stem| {
                        let path = dir.join(format!("{}.rs", stem));
                        if path.exists() {
                            return Err("File already exists".to_string());
                        }
                        std::fs::write(&path, "// new script\n").map_err(|e| e.to_string())?;
                        Ok(path)
                    }),
            );
            reg.register(
                magic_core::AssetTypeDescriptor::new("toml", "TOML Data", egui_phosphor::regular::FILE_TEXT)
                    .category("Data")
                    .color(eframe::egui::Color32::from_rgb(56, 189, 248))
                    .folders(["assets", "editor", "runtime"])
                    .default_name("new_data")
                    .on_create(|dir, stem| {
                        let path = dir.join(format!("{}.toml", stem));
                        if path.exists() {
                            return Err("File already exists".to_string());
                        }
                        std::fs::write(&path, "").map_err(|e| e.to_string())?;
                        Ok(path)
                    }),
            );
            reg.register(
                magic_core::AssetTypeDescriptor::new("json", "JSON Data", egui_phosphor::regular::FILE_TEXT)
                    .category("Data")
                    .color(eframe::egui::Color32::from_rgb(56, 189, 248))
                    .folders(["assets", "editor", "runtime"])
                    .default_name("new_data")
                    .on_create(|dir, stem| {
                        let path = dir.join(format!("{}.json", stem));
                        if path.exists() {
                            return Err("File already exists".to_string());
                        }
                        std::fs::write(&path, "{}").map_err(|e| e.to_string())?;
                        Ok(path)
                    }),
            );
        }


        let tx = self.window_manager.tx.clone();
        self.menu_registry.register_action(
            &["window"],
            "Open Oracle Logs",
            Box::new(move || {
                let _ = tx.send("oracle_logs".to_string());
            }),
        );
        let tx = self.window_manager.tx.clone();
        self.menu_registry.register_action(
            &["window"],
            "Open Inspector",
            Box::new(move || {
                let _ = tx.send("inspector".to_string());
            }),
        );
        let tx = self.window_manager.tx.clone();
        self.menu_registry.register_action(
            &["window"],
            "Open Grimoire",
            Box::new(move || {
                let _ = tx.send("grimoire".to_string());
            }),
        );
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
                    magic_core::request_grimoire_refresh();
                }
                CompilerMessage::CompileError(err) => {
                    self.compiler_manager.set_error(&err);
                    magic_core::oracle_error!("HotReload", "Compile error: {}", err);
                }
            }
        }


        while let Ok(window_id) = self.window_manager.rx.try_recv() {
            self.window_manager.open_window(&self.windows, &window_id);
        }


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
                            .map(|d| d.as_secs())
                            .unwrap_or(0)
                    );
                    self.window_manager.save_layout_named(&name, is_global);
                } else {
                    self.window_manager.save_layout_named(&name, is_global);
                }
                self.window_manager.set_active_layout(&name);
                self.current_layout_name = Some(name.clone());
                self.window_manager.layout_dirty = true;
            }
            LayoutAction::Load { name, is_global } => {
                self.window_manager
                    .load_layout_named(&self.windows, &name, is_global);
                self.window_manager.set_active_layout(&name);
                self.current_layout_name = Some(name.clone());
                self.window_manager.layout_dirty = true;
            }
            LayoutAction::Delete { name, is_global } => {
                self.window_manager.delete_layout(&name, is_global);
                if self.current_layout_name.as_ref() == Some(&name.to_string()) {
                    self.current_layout_name = None;
                }
                self.window_manager.layout_dirty = true;
            }
            LayoutAction::None => {}
        }


        self.window_manager.render_windows(ui, &mut self.windows);
    }

    fn on_exit(&mut self) {
        let mut states: Vec<String> = Vec::new();
        let ids: Vec<u64> = self.window_manager.window_instances.iter_ids().collect();
        for instance_id in ids {
            if let Some(instance) = self.window_manager.window_instances.get_mut(instance_id) {
                if instance.window_id == "grimoire" {
                    if let Some(state) = instance.window.save_state() {
                        states.push(state);
                    }
                }
            }
        }
        if !states.is_empty() {
            let _ = std::fs::create_dir_all("./.magic");
            let _ = std::fs::write("./.magic/grimoire_state.json", serde_json::to_string(&states).unwrap_or_default());
        }
    }
}