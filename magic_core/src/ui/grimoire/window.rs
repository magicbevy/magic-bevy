use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};

static GRIMOIRE_REFRESH_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn request_grimoire_refresh() {
    GRIMOIRE_REFRESH_REQUESTED.store(true, Ordering::Relaxed);
}

pub fn grimoire_refresh_requested() -> bool {
    GRIMOIRE_REFRESH_REQUESTED.load(Ordering::Relaxed)
}

pub fn clear_grimoire_refresh() {
    GRIMOIRE_REFRESH_REQUESTED.store(false, Ordering::Relaxed);
}
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use eframe::egui::{self, Color32, Key, RichText, Sense, Stroke, Vec2};

use super::types::{PackageInfo, ViewMode};
use magic_api::api::assets::global_asset_registry;
use magic_api::api::window::{EditorWindow, WindowId};

const COLOR_SUCCESS: Color32 = Color32::from_rgb(34, 197, 94);

#[derive(Clone, Debug)]
struct DirEntryItem {
    path: PathBuf,
    name: String,
    is_dir: bool,
    package_info: Option<PackageInfo>,
}

#[derive(Clone, Debug, Default)]
struct CreatePackageForm {
    category: String,
    vendor: String,
    name: String,
    target_editor: bool,
    target_runtime: bool,
}

impl CreatePackageForm {
    fn new_with_defaults() -> Self {
        Self {
            category: "gameplay".to_string(),
            vendor: "local".to_string(),
            name: "new_package".to_string(),
            target_editor: true,
            target_runtime: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ClipboardOp {
    Copy,
    Cut,
}

fn pick_zip_file_native() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let script = "Add-Type -AssemblyName System.Windows.Forms; \
                      $f = New-Object System.Windows.Forms.OpenFileDialog; \
                      $f.Filter = 'ZIP Archives (*.zip)|*.zip|All Files (*.*)|*.*'; \
                      $f.Title = 'Select Package ZIP Archive'; \
                      if ($f.ShowDialog() -eq 'OK') { Write-Output $f.FileName }";
        if let Ok(output) = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output()
        {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let script =
            r#"POSIX path of (choose file of type {"zip"} with prompt "Select Package ZIP Archive:")"#;
        if let Ok(output) = Command::new("osascript").args(["-e", script]).output() {
            let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path_str.is_empty() {
                return Some(PathBuf::from(path_str));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = Command::new("zenity")
            .args([
                "--file-selection",
                "--title=Select Package ZIP Archive",
                "--file-filter=ZIP Archives (*.zip) | *.zip",
            ])
            .output()
        {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    return Some(PathBuf::from(path_str));
                }
            }
        }

        if let Ok(output) = Command::new("kdialog")
            .args([
                "--getopenfilename",
                ".",
                "*.zip|ZIP Archives (*.zip)",
                "--title",
                "Select Package ZIP Archive",
            ])
            .output()
        {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    return Some(PathBuf::from(path_str));
                }
            }
        }
    }

    None
}

fn copy_path_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    if src.is_dir() {
        fs::create_dir_all(dst)?;
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let child_src = entry.path();
            let child_dst = dst.join(entry.file_name());
            copy_path_recursive(&child_src, &child_dst)?;
        }
        Ok(())
    } else {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst)?;
        Ok(())
    }
}

pub struct GrimoireWindow {
    packages_root: PathBuf,
    current_dir: PathBuf,
    entries: Vec<DirEntryItem>,
    packages: Vec<PackageInfo>,
    view_mode: ViewMode,
    tile_size: f32,
    search_query: String,

    selected_paths: HashSet<PathBuf>,
    last_selected: Option<PathBuf>,
    context_target: Option<PathBuf>,

    clipboard_paths: Vec<PathBuf>,
    clipboard_op: Option<ClipboardOp>,

    dragging_paths: Vec<PathBuf>,

    active_package_modal: Option<PackageInfo>,
    delete_confirm_package: Option<PackageInfo>,
    show_create_modal: bool,
    create_form: CreatePackageForm,
    show_url_modal: bool,
    install_url: String,

    show_new_folder_modal: Option<PathBuf>,
    new_folder_name: String,
    rename_target: Option<PathBuf>,
    rename_new_name: String,
    rename_edit_clicked: bool,
    rename_edit_rect: Option<egui::Rect>,
    delete_files_confirm: Vec<PathBuf>,
    create_asset_modal: Option<(PathBuf, String, String)>,
    new_asset_name: String,

    status_message: Option<(String, bool)>,

    pkg_version_input: String,
    pkg_versions: Vec<String>,

    discover_cb: Option<Box<dyn Fn() -> Vec<PackageInfo> + Send + Sync>>,
    install_zip_cb: Option<Box<dyn Fn(String) -> Result<String, String> + Send + Sync>>,
    install_url_cb: Option<Box<dyn Fn(String) -> Result<String, String> + Send + Sync>>,
    versions_cb: Option<Box<dyn Fn(String) -> Result<Vec<String>, String> + Send + Sync>>,
    install_version_cb: Option<Box<dyn Fn(String, String) -> Result<String, String> + Send + Sync>>,
    remove_cb: Option<Box<dyn Fn(String) -> Result<String, String> + Send + Sync>>,
    set_status_cb: Option<Box<dyn Fn(String, bool) -> Result<String, String> + Send + Sync>>,
    bump_cb: Option<Box<dyn Fn(String) -> Result<String, String> + Send + Sync>>,
    sync_cb: Option<Box<dyn Fn() -> Result<String, String> + Send + Sync>>,
    update_all_cb: Option<Box<dyn Fn() -> Result<String, String> + Send + Sync>>,
    init_cb: Option<Box<dyn Fn(String, String, String, bool, bool) -> Result<String, String> + Send + Sync>>,
}

impl GrimoireWindow {
    pub fn new() -> Self {
        let packages_root = PathBuf::from("packages");
        let _ = fs::create_dir_all(&packages_root);
        let root_canon = packages_root
            .canonicalize()
            .unwrap_or_else(|_| packages_root.clone());

        let mut win = Self {
            packages_root: root_canon.clone(),
            current_dir: root_canon,
            entries: Vec::new(),
            packages: Vec::new(),
            view_mode: ViewMode::Grid,
            tile_size: 88.0,
            search_query: String::new(),
            selected_paths: HashSet::new(),
            last_selected: None,
            context_target: None,
            clipboard_paths: Vec::new(),
            clipboard_op: None,
            dragging_paths: Vec::new(),
            active_package_modal: None,
            delete_confirm_package: None,
            show_create_modal: false,
            create_form: CreatePackageForm::new_with_defaults(),
            show_url_modal: false,
            install_url: String::new(),
            show_new_folder_modal: None,
            new_folder_name: String::new(),
            rename_target: None,
            rename_new_name: String::new(),
            rename_edit_clicked: false,
            rename_edit_rect: None,
            delete_files_confirm: Vec::new(),
            create_asset_modal: None,
            new_asset_name: String::new(),
            status_message: None,
            pkg_version_input: String::new(),
            pkg_versions: Vec::new(),
            discover_cb: None,
            install_zip_cb: None,
            install_url_cb: None,
            versions_cb: None,
            install_version_cb: None,
            remove_cb: None,
            set_status_cb: None,
            bump_cb: None,
            sync_cb: None,
            update_all_cb: None,
            init_cb: None,
        };
        win.refresh();
        win
    }

    pub fn with_discover<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Vec<PackageInfo> + Send + Sync + 'static,
    {
        self.discover_cb = Some(Box::new(f));
        self
    }

    pub fn with_install_zip<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Result<String, String> + Send + Sync + 'static,
    {
        self.install_zip_cb = Some(Box::new(f));
        self
    }

    pub fn with_install_url<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Result<String, String> + Send + Sync + 'static,
    {
        self.install_url_cb = Some(Box::new(f));
        self
    }

    pub fn with_update_all<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        self.update_all_cb = Some(Box::new(f));
        self
    }

    pub fn with_versions<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Result<Vec<String>, String> + Send + Sync + 'static,
    {
        self.versions_cb = Some(Box::new(f));
        self
    }

    pub fn with_install_version<F>(mut self, f: F) -> Self
    where
        F: Fn(String, String) -> Result<String, String> + Send + Sync + 'static,
    {
        self.install_version_cb = Some(Box::new(f));
        self
    }

    pub fn with_remove<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Result<String, String> + Send + Sync + 'static,
    {
        self.remove_cb = Some(Box::new(f));
        self
    }

    pub fn with_set_status<F>(mut self, f: F) -> Self
    where
        F: Fn(String, bool) -> Result<String, String> + Send + Sync + 'static,
    {
        self.set_status_cb = Some(Box::new(f));
        self
    }

    pub fn with_bump<F>(mut self, f: F) -> Self
    where
        F: Fn(String) -> Result<String, String> + Send + Sync + 'static,
    {
        self.bump_cb = Some(Box::new(f));
        self
    }

    pub fn with_sync<F>(mut self, f: F) -> Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static,
    {
        self.sync_cb = Some(Box::new(f));
        self
    }

    pub fn with_init<F>(mut self, f: F) -> Self
    where
        F: Fn(String, String, String, bool, bool) -> Result<String, String> + Send + Sync + 'static,
    {
        self.init_cb = Some(Box::new(f));
        self
    }

    pub fn refresh_if_requested(&mut self) {
        if grimoire_refresh_requested() {
            self.refresh();
            clear_grimoire_refresh();
        }
    }

    pub fn refresh(&mut self) {
        if let Some(cb) = &self.discover_cb {
            self.packages = (cb)();
        } else {
            self.packages = Vec::new();
        }
        self.entries.clear();

        if !self.current_dir.exists() {
            self.current_dir = self.packages_root.clone();
        }

        if let Ok(read_dir) = fs::read_dir(&self.current_dir) {
            let mut items: Vec<DirEntryItem> = read_dir
                .filter_map(|e| e.ok())
                .map(|entry| {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    let is_dir = path.is_dir();
                    let package_info = if is_dir {
                        self.find_package_at(&path)
                    } else {
                        None
                    };
                    DirEntryItem {
                        path,
                        name,
                        is_dir,
                        package_info,
                    }
                })
                .collect();

            items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            self.entries = items;
        }
    }




    fn find_package_at(&self, dir: &Path) -> Option<PackageInfo> {
        let dir_canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        self.packages.iter().find(|p| {
            let p_canon = p.path.canonicalize().unwrap_or_else(|_| p.path.clone());
            p_canon == dir_canon
        }).cloned()
    }

    fn find_enclosing_package_for(&self, path: &Path) -> Option<PackageInfo> {
        let mut cursor = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let root_canon = self
            .packages_root
            .canonicalize()
            .unwrap_or_else(|_| self.packages_root.clone());

        loop {
            if !cursor.starts_with(&root_canon) || cursor == root_canon {
                break;
            }
            if cursor.join("package.toml").exists() {
                if let Some(pkg) = self.find_package_at(&cursor) {
                    return Some(pkg);
                }
            }
            if !cursor.pop() {
                break;
            }
        }
        None
    }

    fn current_enclosing_package(&self) -> Option<PackageInfo> {
        self.find_enclosing_package_for(&self.current_dir)
    }

    fn get_target_package(&self) -> Option<PackageInfo> {
        if let Some(target) = &self.context_target {
            if let Some(pkg) = self.find_package_at(target) {
                return Some(pkg);
            }
            if let Some(pkg) = self.find_enclosing_package_for(target) {
                return Some(pkg);
            }
        }
        if let Some(sel) = &self.last_selected {
            if let Some(pkg) = self.find_package_at(sel) {
                return Some(pkg);
            }
        }
        self.current_enclosing_package()
    }

    fn can_write_into_dir(&self, dir: &Path) -> bool {
        let Some(pkg) = self.find_enclosing_package_for(dir) else {
            return false;
        };
        let pkg_canon = pkg.path.canonicalize().unwrap_or_else(|_| pkg.path.clone());
        let dir_canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());

        let Ok(rel) = dir_canon.strip_prefix(&pkg_canon) else {
            return false;
        };

        let mut comps = rel.components();
        let Some(first) = comps.next() else {
            return false;
        };

        let section = first.as_os_str().to_string_lossy();
        matches!(section.as_ref(), "assets" | "editor" | "runtime")
    }

    fn can_modify_item(&self, item_path: &Path) -> bool {
        if Self::is_protected_item(item_path) {
            return false;
        }
        let Some(pkg) = self.find_enclosing_package_for(item_path) else {
            return false;
        };
        let pkg_canon = pkg.path.canonicalize().unwrap_or_else(|_| pkg.path.clone());
        let item_canon = item_path
            .canonicalize()
            .unwrap_or_else(|_| item_path.to_path_buf());

        let Ok(rel) = item_canon.strip_prefix(&pkg_canon) else {
            return false;
        };

        let comps: Vec<_> = rel.components().collect();
        if comps.len() < 2 {
            return false;
        }

        let section = comps[0].as_os_str().to_string_lossy();
        matches!(section.as_ref(), "assets" | "editor" | "runtime")
    }

    fn is_protected_item(path: &Path) -> bool {

        if !path.to_string_lossy().contains("packages") {
            return false;
        }
        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        if name == "lib.rs" && path.parent().and_then(|p| p.file_name()).and_then(|s| s.to_str()) == Some("src") {
            return true;
        }
        if name == "src" && path.is_dir() {
            return true;
        }

        if name == "Cargo.toml" {
            return true;
        }

        if name == "package.toml" {
            return true;
        }
        false
    }

    fn open_in_os_file_manager(path: &Path) {
        let dir = if path.is_dir() { path.to_path_buf() } else {
            path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| path.to_path_buf())
        };
        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(&dir).spawn();
        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open").arg(&dir).spawn();
        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("explorer").arg(&dir).spawn();
    }

    fn section_for_dir(&self, dir: &Path) -> Option<String> {
        let pkg = self.find_enclosing_package_for(dir)?;
        let pkg_canon = pkg.path.canonicalize().unwrap_or_else(|_| pkg.path.clone());
        let dir_canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let rel = dir_canon.strip_prefix(&pkg_canon).ok()?;
        let first = rel.components().next()?.as_os_str().to_string_lossy().to_string();
        if matches!(first.as_str(), "assets" | "editor" | "runtime") {
            Some(first)
        } else {
            None
        }
    }

    fn subdir_for_dir(&self, dir: &Path) -> Option<String> {
        let pkg = self.find_enclosing_package_for(dir)?;
        let pkg_canon = pkg.path.canonicalize().unwrap_or_else(|_| pkg.path.clone());
        let dir_canon = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        let rel = dir_canon.strip_prefix(&pkg_canon).ok()?;
        let mut comps = rel.components();
        let _first = comps.next()?;
        comps.next().map(|c| c.as_os_str().to_string_lossy().to_string())
    }

    fn svg_texture(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        let key = egui::Id::new(("svg_tex", path, 128u32));
        if let Some(tex) = ctx.data_mut(|d| d.get_persisted::<egui::TextureHandle>(key)) {
            return Some(tex);
        }
        let bytes = std::fs::read(path).ok()?;
        let svg_str = std::str::from_utf8(&bytes).ok()?;
        let opts = usvg::Options::default();
        let tree = usvg::Tree::from_str(svg_str, &opts).ok()?;
        let size = tree.size();
        let target = 128u32;
        let scale = target as f32 / size.width().max(size.height());
        let w = (size.width() * scale).ceil().max(1.0) as u32;
        let h = (size.height() * scale).ceil().max(1.0) as u32;
        let mut pixmap = tiny_skia::Pixmap::new(w, h)?;
        let transform = tiny_skia::Transform::from_scale(scale, scale);
        resvg::render(&tree, transform, &mut pixmap.as_mut());
        let pixels: Vec<egui::Color32> = pixmap
            .pixels()
            .iter()
            .map(|p| egui::Color32::from_rgba_premultiplied(p.red(), p.green(), p.blue(), p.alpha()))
            .collect();
        let image = egui::ColorImage {
            size: [w as usize, h as usize],
            source_size: egui::Vec2::new(w as f32, h as f32),
            pixels,
        };
        let tex = ctx.load_texture(path, image, egui::TextureOptions::default());
        ctx.data_mut(|d| d.insert_persisted(key, tex.clone()));
        Some(tex)
    }

    fn svg_path_for_entry(entry: &DirEntryItem) -> Option<&'static str> {
        if entry.is_dir {
            if entry.package_info.is_some() {
                return Some("assets/icons/grimoire/package_box.svg");
            }
            let empty = std::fs::read_dir(&entry.path)
                .map(|mut d| d.next().is_none())
                .unwrap_or(true);
            if empty {
                Some("assets/icons/grimoire/folder_empty.svg")
            } else {
                Some("assets/icons/grimoire/folder_full.svg")
            }
        } else {
            let ext = entry.path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            match ext.as_str() {
                "rs" => Some("assets/icons/grimoire/rust_script.svg"),
                "toml" => Some("assets/icons/grimoire/config_file.svg"),
                "json" => Some("assets/icons/grimoire/data_json.svg"),
                "wav" | "ogg" | "mp3" => Some("assets/icons/grimoire/audio_file.svg"),
                _ => Some("assets/icons/grimoire/unknown_file.svg"),
            }
        }
    }




    fn select_single_item(&mut self, path: PathBuf) {
        self.selected_paths.clear();
        self.selected_paths.insert(path.clone());
        self.last_selected = Some(path.clone());

        crate::ui::inspector::set_inspector_target_file(path.clone());
        if let Ok(mut reg) = global_asset_registry().lock() {
            reg.set_selected_path(Some(path));
        }
    }

    fn open_item(&mut self, path: &Path) {
        if path.is_dir() {
            self.current_dir = path.to_path_buf();
            self.selected_paths.clear();
            self.last_selected = None;
            crate::ui::inspector::clear_inspector_target();
            self.refresh();
            return;
        }

        let mut opened_by_package = false;
        if let Ok(reg) = global_asset_registry().lock() {
            if let Some(desc) = reg.get_by_path(path) {
                if let Some(open_fn) = &desc.on_open {
                    (open_fn)(path);
                    opened_by_package = true;
                    self.status_message = Some((
                        format!("Opened '{}' via {}", path.display(), desc.display_name),
                        false,
                    ));
                }
            }
        }

        if !opened_by_package {
            self.select_single_item(path.to_path_buf());
        }
    }

    fn move_items_into_dir(&mut self, items: &[PathBuf], target_dir: &Path) {
        if !self.can_write_into_dir(target_dir) {
            self.status_message = Some((
                "Move denied: Only allowed inside 'assets', 'editor', or 'runtime'.".to_string(),
                true,
            ));
            return;
        }

        let target_canon = target_dir
            .canonicalize()
            .unwrap_or_else(|_| target_dir.to_path_buf());
        let mut moved_count = 0usize;

        for src in items {
            if !self.can_modify_item(src) {
                continue;
            }
            let src_canon = src.canonicalize().unwrap_or_else(|_| src.clone());
            if target_canon.starts_with(&src_canon) {
                continue;
            }
            if let Some(file_name) = src.file_name() {
                let dst = target_dir.join(file_name);
                if dst == *src {
                    continue;
                }
                if fs::rename(src, &dst).is_ok() {
                    moved_count += 1;
                }
            }
        }

        if moved_count > 0 {
            self.selected_paths.clear();
            self.last_selected = None;
            crate::ui::inspector::clear_inspector_target();
            self.refresh();
            self.status_message = Some((format!("Moved {} item(s).", moved_count), false));
        }
    }

    fn paste_clipboard_into(&mut self, target_dir: &Path) {
        if !self.can_write_into_dir(target_dir) || self.clipboard_paths.is_empty() {
            return;
        }

        let Some(op) = self.clipboard_op else {
            return;
        };

        let paths = self.clipboard_paths.clone();
        match op {
            ClipboardOp::Cut => {
                self.move_items_into_dir(&paths, target_dir);
                self.clipboard_paths.clear();
                self.clipboard_op = None;
            }
            ClipboardOp::Copy => {
                let mut copied = 0usize;
                for src in &paths {
                    if !self.can_modify_item(src) {
                        continue;
                    }
                    if let Some(name) = src.file_name() {
                        let mut dst = target_dir.join(name);
                        if dst.exists() {
                            let stem = src
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("item");
                            let ext = src
                                .extension()
                                .and_then(|s| s.to_str())
                                .map(|e| format!(".{}", e))
                                .unwrap_or_default();
                            dst = target_dir.join(format!("{}_copy{}", stem, ext));
                        }
                        if copy_path_recursive(src, &dst).is_ok() {
                            copied += 1;
                        }
                    }
                }
                if copied > 0 {
                    self.refresh();
                    self.status_message = Some((format!("Copied {} item(s).", copied), false));
                }
            }
        }
    }

    fn handle_os_dropped_files(&mut self, ui: &mut egui::Ui) {
        let hovered_files = ui.ctx().input(|i| i.raw.hovered_files.clone());
        let dropped_files = ui.ctx().input(|i| i.raw.dropped_files.clone());

        let allowed = self.can_write_into_dir(&self.current_dir);

        if !hovered_files.is_empty() {
            let rect = ui.max_rect();
            let color = if allowed {
                ui.visuals().selection.stroke.color
            } else {
                ui.visuals().error_fg_color
            };
            ui.painter().rect_filled(
                rect,
                6.0,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28),
            );
            ui.painter()
                .rect_stroke(rect, 6.0, Stroke::new(2.0, color), egui::StrokeKind::Inside);

            let msg = if allowed {
                format!(
                    " Drop {} file(s) to import into '{}'",
                    hovered_files.len(),
                    self.current_dir
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("folder")
                )
            } else {
                " Dropping files is only allowed inside 'assets', 'editor', or 'runtime' folders!"
                    .to_string()
            };

            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                msg,
                egui::FontId::proportional(15.0),
                color,
            );
        }

        if !dropped_files.is_empty() {
            if !allowed {
                self.status_message = Some((
                    "Cannot import here: Navigate inside 'assets', 'editor', or 'runtime' first."
                        .to_string(),
                    true,
                ));
                return;
            }

            let mut imported = 0usize;
            for file in dropped_files {
                let src_path = file.path().to_path_buf();
                if let Some(name) = src_path.file_name() {
                    let dst = self.current_dir.join(name);
                    if copy_path_recursive(&src_path, &dst).is_ok() {
                        imported += 1;
                    }
                }
            }

            if imported > 0 {
                self.refresh();
                self.status_message =
                    Some((format!("Imported {} item(s) from OS.", imported), false));
            }
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ui: &mut egui::Ui) {
        if self.show_create_modal
            || self.show_url_modal
            || self.show_new_folder_modal.is_some()
            || self.rename_target.is_some()
            || self.create_asset_modal.is_some()
        {
            return;
        }

        let editable_selection: Vec<PathBuf> = self
            .selected_paths
            .iter()
            .filter(|p| self.can_modify_item(p))
            .cloned()
            .collect();


        if ui.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::C))
            && !editable_selection.is_empty()
        {
            self.clipboard_paths = editable_selection.clone();
            self.clipboard_op = Some(ClipboardOp::Copy);
            self.status_message = Some((
                format!("Copied {} item(s) to clipboard.", editable_selection.len()),
                false,
            ));
        }


        if ui.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::X))
            && !editable_selection.is_empty()
        {
            self.clipboard_paths = editable_selection.clone();
            self.clipboard_op = Some(ClipboardOp::Cut);
            self.status_message = Some((
                format!("Cut {} item(s). Ready to move.", editable_selection.len()),
                false,
            ));
        }


        if ui.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::V))
            && self.can_write_into_dir(&self.current_dir)
        {
            let target = self.current_dir.clone();
            self.paste_clipboard_into(&target);
        }


        if ui.input(|i| i.key_pressed(Key::F2)) && editable_selection.len() == 1 {
            let item = editable_selection[0].clone();
            self.rename_new_name = item
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            self.rename_target = Some(item);
        }


        if ui.input(|i| i.key_pressed(Key::Delete)) && !editable_selection.is_empty() {
            self.delete_files_confirm = editable_selection;
        }
    }

    fn trigger_native_zip_install(&mut self) {
        if let Some(zip_path) = pick_zip_file_native() {
            if let Some(cb) = &self.install_zip_cb {
                match (cb)(zip_path.to_string_lossy().to_string()) {
                    Ok(msg) => {
                        self.status_message = Some((msg, false));
                        self.refresh();
                    }
                    Err(err) => {
                        self.status_message = Some((err, true));
                    }
                }
            }
        }
    }

    fn resolve_item_visuals(
        &self,
        entry: &DirEntryItem,
        ui: &egui::Ui,
    ) -> (String, Option<String>, Color32) {
        if entry.is_dir {
            if let Some(pkg) = &entry.package_info {
                let color = if pkg.enabled {
                    COLOR_SUCCESS
                } else {
                    ui.visuals().weak_text_color()
                };
                return (egui_phosphor::regular::PACKAGE.to_string(), Some(format!("v{}", pkg.version)), color);
            }

            return (egui_phosphor::regular::FOLDER.to_string(), None, ui.visuals().hyperlink_color);
        }

        if let Ok(reg) = global_asset_registry().lock() {
            if let Some(desc) = reg.get_by_path(&entry.path) {
                return (
                    desc.icon.clone(),
                    Some(desc.display_name.clone()),
                    desc.color,
                );
            }
        }

        let ext = entry
            .path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "rs" => (egui_phosphor::regular::FILE_RS.to_string(), Some("Rust".to_string()), Color32::from_rgb(249, 115, 22)),
            "toml" => (egui_phosphor::regular::GEAR.to_string(), Some("TOML".to_string()), ui.visuals().hyperlink_color),
            "png" | "jpg" | "jpeg" | "webp" => (
                egui_phosphor::regular::IMAGE.to_string(),
                Some("Image".to_string()),
                Color32::from_rgb(56, 189, 248),
            ),
            "glb" | "gltf" | "obj" | "fbx" => (
                egui_phosphor::regular::CUBE.to_string(),
                Some("3D Mesh".to_string()),
                Color32::from_rgb(34, 197, 94),
            ),
            "wav" | "ogg" | "mp3" => (
                egui_phosphor::regular::SPEAKER_HIGH.to_string(),
                Some("Audio".to_string()),
                Color32::from_rgb(245, 185, 66),
            ),
            "wgsl" | "glsl" => (
                egui_phosphor::regular::SPARKLE.to_string(),
                Some("Shader".to_string()),
                Color32::from_rgb(168, 85, 247),
            ),
            _ => (egui_phosphor::regular::FILE.to_string(), None, ui.visuals().weak_text_color()),
        }
    }




    fn draw_top_toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(egui::Margin::symmetric(6, 4))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(4.0, 4.0);
                    ui.spacing_mut().button_padding = egui::Vec2::new(6.0, 3.0);
                    if ui
                        .add(egui::Button::new("+").small().min_size(egui::Vec2::new(26.0, 20.0)))
                        .on_hover_text("Create a new package structure")
                        .clicked()
                    {
                        self.create_form = CreatePackageForm::new_with_defaults();
                        self.show_create_modal = true;
                    }

                    ui.menu_button(egui_phosphor::regular::DOWNLOAD, |ui| {
                        ui.set_min_width(180.0);
                        if ui.small_button(" From ZIP (OS)...").clicked() {
                            ui.close();
                            self.trigger_native_zip_install();
                        }
                        if ui.small_button(" From URL...").clicked() {
                            ui.close();
                            self.install_url.clear();
                            self.show_url_modal = true;
                        }
                    });

                    if ui.small_button(egui_phosphor::regular::ARROWS_CLOCKWISE).on_hover_text("Sync").clicked() {
                        if let Some(cb) = &self.sync_cb {
                            let _ = (cb)();
                        }
                        self.refresh();
                    }

                    if ui.small_button("Update").on_hover_text("Update all packages").clicked() {
                        if let Some(cb) = &self.update_all_cb {
                            let _ = (cb)();
                        }
                        self.refresh();
                    }

                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text(" Search...")
                            .desired_width(120.0),
                    );
                    if !self.search_query.is_empty() && ui.small_button(egui_phosphor::regular::X).clicked() {
                        self.search_query.clear();
                    }

                });
            });
    }

    fn draw_breadcrumb_bar(&mut self, ui: &mut egui::Ui) {
        let enclosing_pkg = self.current_enclosing_package();

        egui::Frame::group(ui.style())
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(4, 2))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing = egui::Vec2::new(2.0, 0.0);
                    if ui.small_button(egui_phosphor::regular::ARROW_LEFT).on_hover_text("Up").clicked() {
                        if let Some(parent) = self.current_dir.parent() {
                            if parent.starts_with(&self.packages_root) {
                                self.current_dir = parent.to_path_buf();
                                self.refresh();
                            }
                        }
                    }

                    if ui
                        .add(egui::Label::new("packages").sense(egui::Sense::click()))
                        .clicked()
                    {
                        self.current_dir = self.packages_root.clone();
                        self.refresh();
                    }

                    let rel_components: Vec<String> = self
                        .current_dir
                        .strip_prefix(&self.packages_root)
                        .ok()
                        .map(|rel| {
                            rel.components()
                                .map(|c| c.as_os_str().to_string_lossy().to_string())
                                .collect()
                        })
                        .unwrap_or_default();

                    const MAX_CRUMBS: usize = 3;
                    let omitted = if rel_components.len() > MAX_CRUMBS {
                        rel_components.len() - MAX_CRUMBS
                    } else {
                        0
                    };

                    if !rel_components.is_empty() {
                        let mut accumulated = self.packages_root.clone();
                        for part in &rel_components[..omitted] {
                            accumulated.push(part);
                        }

                        if omitted > 0 {
                            ui.weak("›");
                            ui.weak("…");
                        }

                        for part in &rel_components[omitted..] {
                            accumulated.push(part);
                            let target_path = accumulated.clone();

                            ui.weak("›");
                            let lbl = ui.add(egui::Label::new(part).sense(egui::Sense::click()));
                            if lbl.clicked() {
                                self.current_dir = target_path.clone();
                                self.refresh();
                            }


                            if !self.dragging_paths.is_empty()
                                && self.can_write_into_dir(&target_path)
                                && ui.rect_contains_pointer(lbl.rect)
                            {
                                ui.painter().rect_stroke(
                                    lbl.rect,
                                    2.0,
                                    ui.visuals().selection.stroke,
                                    egui::StrokeKind::Inside,
                                );
                                if ui.input(|i| i.pointer.any_released()) {
                                    let dragged = std::mem::take(&mut self.dragging_paths);
                                    self.move_items_into_dir(&dragged, &target_path);
                                }
                            }
                        }
                    }

                    if let Some(pkg) = enclosing_pkg {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button(egui_phosphor::regular::FILE_TEXT).on_hover_text("Package Details").clicked() {
                                self.active_package_modal = Some(pkg.clone());
                            }
                        });
                    }
                });
            });
    }

    fn visible_entries(&self) -> Vec<DirEntryItem> {
        let q = self.search_query.trim().to_lowercase();
        self.entries
            .iter()
            .filter(|e| q.is_empty() || e.name.to_lowercase().contains(&q))
            .cloned()
            .collect()
    }

    fn populate_context_menu(&mut self, ui: &mut egui::Ui, target_item: Option<PathBuf>) {
        ui.set_min_width(210.0);

        let reveal_path = target_item.clone().unwrap_or_else(|| self.current_dir.clone());
        if ui.button("  Open in File Manager").clicked() {
            Self::open_in_os_file_manager(&reveal_path);
            ui.close();
        }
        ui.separator();

        let target_dir = match &target_item {
            Some(p) if p.is_dir() => p.clone(),
            _ => self.current_dir.clone(),
        };

        let dir_writable = self.can_write_into_dir(&target_dir);
        let item_modifiable = target_item
            .as_ref()
            .map(|p| self.can_modify_item(p))
            .unwrap_or(false);

        if let Some(ref item) = target_item {
            let label = if item.is_dir() {
                "  Open Folder"
            } else {
                "  Open / Execute"
            };
            if ui.button(label).clicked() {
                let p = item.clone();
                ui.close();
                self.open_item(&p);
            }
            ui.separator();
        }

        if dir_writable {
            if ui.button("+  New Folder").clicked() {
                let base = "new_folder";
                let mut name = base.to_string();
                let mut i = 2;
                while target_dir.join(&name).exists() {
                    name = format!("{} ({})", base, i);
                    i += 1;
                }
                let new_path = target_dir.join(&name);
                if fs::create_dir_all(&new_path).is_ok() {
                    self.refresh();
                    self.select_single_item(new_path.clone());
                    self.rename_new_name = name;
                    self.rename_target = Some(new_path);
                    self.rename_edit_rect = None;
                }
                ui.close();
            }

            let target_section = self.section_for_dir(&target_dir);
            let target_subdir = self.subdir_for_dir(&target_dir);
            ui.menu_button(format!("{} Create", egui_phosphor::regular::PLUS), |ui| {
                ui.set_min_width(200.0);
                let mut registered_list = Vec::new();
                if let Ok(reg) = global_asset_registry().lock() {
                    for desc in reg.all_types() {
                        if desc.on_create.is_some() {
                            let allowed = match &target_section {
                                Some(sec) => desc.allowed_in(sec) && desc.allowed_in_subfolder(target_subdir.as_deref()),
                                None => false,
                            };
                            if allowed {
                                registered_list.push((
                                    desc.extension.clone(),
                                    desc.display_name.clone(),
                                    desc.icon.clone(),
                                    desc.category.clone(),
                                ));
                            }
                        }
                    }
                }

                if registered_list.is_empty() {
                    ui.weak("No package file creators registered yet.");
                } else {
                    let mut groups: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();
                    let mut ungrouped: Vec<(String, String, String, String)> = Vec::new();
                    for (ext, display_name, icon, category) in registered_list {
                        match category {
                            Some(cat) => groups
                                .entry(cat)
                                .or_default()
                                .push((ext, display_name, icon)),
                            None => ungrouped.push((ext, display_name, icon, String::new())),
                        }
                    }

                    for (ext, display_name, icon, _) in &ungrouped {
                        let label = format!("{}  {} (.{})", icon, display_name, ext);
                        if ui.button(label).clicked() {
                            let base = global_asset_registry().lock().ok()
                                .and_then(|reg| reg.get_by_extension(&ext).and_then(|d| d.default_name.clone()))
                                .unwrap_or_else(|| format!("new_{}", ext));
                            let mut stem = base.clone();
                            let mut i = 2;
                            while target_dir.join(format!("{}.{}", stem, ext)).exists() {
                                stem = format!("{} ({})", base, i);
                                i += 1;
                            }
                            let mut created_path = None;
                            if let Ok(reg) = global_asset_registry().lock() {
                                if let Some(desc) = reg.get_by_extension(&ext) {
                                    if let Some(create_fn) = &desc.on_create {
                                        if let Ok(p) = (create_fn)(&target_dir, &stem) {
                                            created_path = Some(p);
                                        }
                                    }
                                }
                            }
                            if let Some(p) = created_path {
                                self.refresh();
                                self.select_single_item(p.clone());
                                self.rename_new_name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                                self.rename_target = Some(p);
                                self.rename_edit_rect = None;
                            }
                            ui.close();
                        }
                    }

                    if !ungrouped.is_empty() && !groups.is_empty() {
                        ui.separator();
                    }

                    for (cat, items) in &groups {
                        ui.menu_button(format!("{} {}", egui_phosphor::regular::FOLDER, cat), |ui| {
                            ui.set_min_width(180.0);
                            for (ext, display_name, icon) in items {
                                let label = format!("{}  {} (.{})", icon, display_name, ext);
                                if ui.button(label).clicked() {
                                    let base = global_asset_registry().lock().ok()
                                .and_then(|reg| reg.get_by_extension(&ext).and_then(|d| d.default_name.clone()))
                                .unwrap_or_else(|| format!("new_{}", ext));
                                    let mut stem = base.clone();
                                    let mut i = 2;
                                    while target_dir.join(format!("{}.{}", stem, ext)).exists() {
                                        stem = format!("{} ({})", base, i);
                                        i += 1;
                                    }
                                    let mut created_path = None;
                                    if let Ok(reg) = global_asset_registry().lock() {
                                        if let Some(desc) = reg.get_by_extension(&ext) {
                                            if let Some(create_fn) = &desc.on_create {
                                                if let Ok(p) = (create_fn)(&target_dir, &stem) {
                                                    created_path = Some(p);
                                                }
                                            }
                                        }
                                    }
                                    if let Some(p) = created_path {
                                        self.refresh();
                                        self.select_single_item(p.clone());
                                        self.rename_new_name = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                                        self.rename_target = Some(p);
                                        self.rename_edit_rect = None;
                                    }
                                    ui.close();
                                }
                            }
                        });
                    }
                }
            });

            ui.separator();
        }

        if item_modifiable {
            if let Some(ref item) = target_item {
                let active_targets: Vec<PathBuf> = if self.selected_paths.contains(item) {
                    self.selected_paths
                        .iter()
                        .filter(|p| self.can_modify_item(p))
                        .cloned()
                        .collect()
                } else {
                    vec![item.clone()]
                };

                if active_targets.len() == 1 && !Self::is_protected_item(&active_targets[0]) {
                    if ui.button("  Rename (F2)").clicked() {
                        let t = active_targets[0].clone();
                        self.rename_new_name = t
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("")
                            .to_string();
                        self.rename_target = Some(t);
                        ui.close();
                    }
                }

                if ui
                    .button(format!("  Copy ({})", active_targets.len()))
                    .clicked()
                {
                    self.clipboard_paths = active_targets.clone();
                    self.clipboard_op = Some(ClipboardOp::Copy);
                    ui.close();
                }

                if ui
                    .button(format!("  Cut / Move ({})", active_targets.len()))
                    .clicked()
                {
                    self.clipboard_paths = active_targets.clone();
                    self.clipboard_op = Some(ClipboardOp::Cut);
                    ui.close();
                }

                let has_protected = active_targets.iter().any(|p| Self::is_protected_item(p));
                if !has_protected {
                    let del_label = RichText::new(format!("  Delete ({})", active_targets.len()))
                        .color(ui.visuals().error_fg_color);
                    if ui.button(del_label).clicked() {
                        self.delete_files_confirm = active_targets;
                        ui.close();
                    }
                }

                ui.separator();
            }
        }

        if dir_writable && !self.clipboard_paths.is_empty() {
            let op_name = match self.clipboard_op {
                Some(ClipboardOp::Cut) => "Move Here",
                _ => "Paste Copy Here",
            };
            if ui
                .button(format!(
                    "  {} ({} item(s))",
                    op_name,
                    self.clipboard_paths.len()
                ))
                .clicked()
            {
                ui.close();
                self.paste_clipboard_into(&target_dir);
            }
            ui.separator();
        }

        if let Some(pkg) = self.get_target_package() {
            ui.label(
                RichText::new(format!(" Package: {}", pkg.name))
                    .size(11.0)
                    .color(ui.visuals().weak_text_color()),
            );

            if ui.button("  Show Package Details").clicked() {
                self.active_package_modal = Some(pkg.clone());
                ui.close();
            }

            let toggle_text = if pkg.enabled {
                "  Disable Package"
            } else {
                "▶  Enable Package"
            };
            if ui.button(toggle_text).clicked() {
                if let Some(cb) = &self.set_status_cb {
                    let id = pkg.id.clone();
                    if let Err(err) = (cb)(id, !pkg.enabled) {
                        self.status_message = Some((err, true));
                    } else {
                        self.refresh();
                    }
                }
                ui.close();
            }

            if target_item.as_ref() == Some(&pkg.path) {
                let del_pkg = RichText::new("  Delete Package...")
                    .color(ui.visuals().error_fg_color);
                if ui.button(del_pkg).clicked() {
                    self.delete_confirm_package = Some(pkg);
                    ui.close();
                }
            }
            ui.separator();
        }

        if target_item.is_none() {
            if ui.button("  New Package...").clicked() {
                self.create_form = CreatePackageForm::new_with_defaults();
                self.show_create_modal = true;
                ui.close();
            }
            if ui.button("  Refresh").clicked() {
                self.refresh();
                ui.close();
            }
        }
    }

    fn handle_item_interaction(
        &mut self,
        ui: &mut egui::Ui,
        entry: &DirEntryItem,
        response: &egui::Response,
    ) {
        if response.drag_started() && self.can_modify_item(&entry.path) {
            if self.selected_paths.contains(&entry.path) {
                self.dragging_paths = self
                    .selected_paths
                    .iter()
                    .filter(|p| self.can_modify_item(p))
                    .cloned()
                    .collect();
            } else {
                self.select_single_item(entry.path.clone());
                self.dragging_paths = vec![entry.path.clone()];
            }
        }

        if entry.is_dir
            && !self.dragging_paths.is_empty()
            && !self.dragging_paths.contains(&entry.path)
            && self.can_write_into_dir(&entry.path)
            && ui.rect_contains_pointer(response.rect)
        {
            ui.painter().rect_stroke(
                response.rect,
                6.0,
                Stroke::new(2.0, COLOR_SUCCESS),
                egui::StrokeKind::Inside,
            );

            if ui.input(|i| i.pointer.any_released()) {
                let dragged = std::mem::take(&mut self.dragging_paths);
                self.move_items_into_dir(&dragged, &entry.path);
            }
        }

        if response.clicked() {
            if self.rename_target.is_some() && self.rename_target.as_ref() != Some(&entry.path) {
                let target = self.rename_target.clone().unwrap();
                let new_name = self.rename_new_name.trim().to_string();
                if !new_name.is_empty() {
                    if let Some(parent) = target.parent() {
                        let new_path = parent.join(&new_name);
                        if fs::rename(&target, &new_path).is_ok() {
                            self.refresh();
                        }
                    }
                }
                self.rename_target = None;
            }
            let modifiers = ui.input(|i| i.modifiers);
            if modifiers.ctrl || modifiers.command {
                if self.selected_paths.contains(&entry.path) {
                    self.selected_paths.remove(&entry.path);
                } else {
                    self.selected_paths.insert(entry.path.clone());
                    self.last_selected = Some(entry.path.clone());
                }
                crate::ui::inspector::set_inspector_target_files(self.selected_paths.iter().cloned().collect());
            } else {
                self.select_single_item(entry.path.clone());
            }
        }

        if response.double_clicked() {
            let path = entry.path.clone();
            self.open_item(&path);
        }

        response.context_menu(|ui| {
            self.context_target = Some(entry.path.clone());
            if !self.selected_paths.contains(&entry.path) {
                self.select_single_item(entry.path.clone());
            }
            self.populate_context_menu(ui, Some(entry.path.clone()));
        });
    }

    fn draw_grid_view(&mut self, ui: &mut egui::Ui, entries: &[DirEntryItem]) {
        let spacing = 10.0;
        let card_w = self.tile_size;
        let card_h = self.tile_size + 28.0;
        let avail_w = ui.available_width().max(card_w);
        let cols = ((avail_w + spacing) / (card_w + spacing)).floor().max(1.0) as usize;

        for row in entries.chunks(cols) {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = Vec2::splat(spacing);

                for entry in row {
                    let is_selected = self.selected_paths.contains(&entry.path);
                    let is_cut = self.clipboard_op == Some(ClipboardOp::Cut)
                        && self.clipboard_paths.contains(&entry.path);

                    let (icon, badge_opt, accent_color) = self.resolve_item_visuals(entry, ui);

                    let fill = if is_selected {
                        ui.visuals().selection.bg_fill
                    } else {
                        ui.visuals().widgets.inactive.bg_fill
                    };

                    let stroke = if is_selected {
                        ui.visuals().selection.stroke
                    } else {
                        ui.visuals().widgets.noninteractive.bg_stroke
                    };

                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::new(card_w, card_h), Sense::click_and_drag());

                    let bg_fill = if response.hovered() && !is_selected {
                        ui.visuals().widgets.hovered.bg_fill
                    } else {
                        fill
                    };

                    ui.painter().rect_filled(rect, 6.0, bg_fill);
                    ui.painter()
                        .rect_stroke(rect, 6.0, stroke, egui::StrokeKind::Inside);

                    let icon_size = if entry.is_dir {
                        (self.tile_size * 0.55).clamp(28.0, 68.0)
                    } else {
                        (self.tile_size * 0.42).clamp(22.0, 54.0)
                    };
                    let icon_pos = egui::pos2(rect.center().x, rect.top() + card_h * 0.36);
                    if let Some(svg_path) = Self::svg_path_for_entry(entry) {
                        if let Some(tex) = Self::svg_texture(ui.ctx(), svg_path) {
                            let img_size = egui::Vec2::splat(icon_size);
                            let img_rect = egui::Rect::from_center_size(icon_pos, img_size);
                            ui.painter().image(
                                tex.id(),
                                img_rect,
                                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                egui::Color32::WHITE,
                            );
                        } else {
                            ui.painter().text(
                                icon_pos,
                                egui::Align2::CENTER_CENTER,
                                &icon,
                                egui::FontId::proportional(icon_size),
                                if is_cut {
                                    ui.visuals().weak_text_color()
                                } else {
                                    ui.visuals().text_color()
                                },
                            );
                        }
                    } else {
                        ui.painter().text(
                            icon_pos,
                            egui::Align2::CENTER_CENTER,
                            &icon,
                            egui::FontId::proportional(icon_size),
                            if is_cut {
                                ui.visuals().weak_text_color()
                            } else {
                                ui.visuals().text_color()
                            },
                        );
                    }

                    if let Some(badge) = badge_opt {
                        if self.tile_size >= 72.0 {
                            let badge_pos = egui::pos2(rect.center().x, rect.top() + card_h * 0.64);
                            ui.painter().text(
                                badge_pos,
                                egui::Align2::CENTER_CENTER,
                                badge,
                                egui::FontId::proportional(10.0),
                                accent_color,
                            );
                        }
                    }

                    let max_chars = ((card_w / 7.2) as usize).max(6);
                    let display_name = if entry.name.chars().count() > max_chars {
                        let truncated: String = entry.name.chars().take(max_chars - 1).collect();
                        format!("{}…", truncated)
                    } else {
                        entry.name.clone()
                    };

                    let text_pos = egui::pos2(rect.center().x, rect.bottom() - 13.0);
                    let is_renaming = self.rename_target.as_ref() == Some(&entry.path);

                    if is_renaming {
                        let edit_rect = egui::Rect::from_center_size(
                            text_pos,
                            egui::Vec2::new(card_w - 6.0, 16.0),
                        );
                        self.rename_edit_rect = Some(edit_rect);
                        let resp = ui.put(
                            edit_rect,
                            egui::TextEdit::singleline(&mut self.rename_new_name)
                                .desired_width(card_w - 6.0)
                                .font(egui::FontId::proportional(11.5)),
                        );
                        if !resp.has_focus() {
                            resp.request_focus();
                        }
                        self.rename_edit_clicked = self.rename_edit_clicked || resp.has_focus();
                        let commit = resp.lost_focus()
                            || ui.input(|i| i.key_pressed(egui::Key::Enter));
                        let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));
                        if commit {
                            if let Some(parent) = entry.path.parent() {
                                let new_name = self.rename_new_name.trim();
                                if !new_name.is_empty() {
                                    let new_path = parent.join(new_name);
                                    if fs::rename(&entry.path, &new_path).is_ok() {
                                        self.refresh();
                                        self.select_single_item(new_path);
                                    }
                                }
                            }
                            self.rename_target = None;
                        }
                        if cancel {
                            self.rename_target = None;
                        }
                    } else {
                        let text_pos = egui::pos2(rect.center().x, rect.bottom() - 13.0);
                        ui.painter().text(
                            text_pos,
                            egui::Align2::CENTER_CENTER,
                            display_name,
                            egui::FontId::proportional(11.5),
                            ui.visuals().text_color(),
                        );
                    }

                    self.handle_item_interaction(ui, entry, &response);
                }
            });
            ui.add_space(spacing);
        }
    }

    fn draw_list_view(&mut self, ui: &mut egui::Ui, entries: &[DirEntryItem]) {
        if self.current_dir != self.packages_root {
            if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
                if parent.starts_with(&self.packages_root) {
                    let resp = ui.selectable_label(false, "  .. (Parent Directory)");
                    if resp.double_clicked() {
                        self.current_dir = parent.clone();
                        self.refresh();
                        return;
                    }
                    if !self.dragging_paths.is_empty()
                        && self.can_write_into_dir(&parent)
                        && ui.rect_contains_pointer(resp.rect)
                    {
                        ui.painter().rect_stroke(
                            resp.rect,
                            4.0,
                            Stroke::new(1.5, COLOR_SUCCESS),
                            egui::StrokeKind::Inside,
                        );
                        if ui.input(|i| i.pointer.any_released()) {
                            let dragged = std::mem::take(&mut self.dragging_paths);
                            self.move_items_into_dir(&dragged, &parent);
                            return;
                        }
                    }
                }
            }
        }

        for (idx, entry) in entries.iter().enumerate() {
            let is_selected = self.selected_paths.contains(&entry.path);
            let (icon, badge_opt, accent_color) = self.resolve_item_visuals(entry, ui);

            let row_bg = if is_selected {
                ui.visuals().selection.bg_fill
            } else if idx % 2 == 1 {
                ui.visuals().faint_bg_color
            } else {
                Color32::TRANSPARENT
            };

            let frame_resp = egui::Frame::new()
                .fill(row_bg)
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if let Some(svg_path) = Self::svg_path_for_entry(entry) {
                            if let Some(tex) = Self::svg_texture(ui.ctx(), svg_path) {
                                ui.add(egui::Image::new(&tex).fit_to_exact_size(egui::Vec2::splat(16.0)));
                            } else {
                                ui.label(RichText::new(icon).size(15.0));
                            }
                        } else {
                            ui.label(RichText::new(icon).size(15.0));
                        }
                        ui.add_space(4.0);
                        if self.rename_target.as_ref() == Some(&entry.path) {
                            let resp = ui.add(egui::TextEdit::singleline(&mut self.rename_new_name).desired_width(220.0));
                            self.rename_edit_rect = Some(resp.rect);
                            if !resp.has_focus() {
                                resp.request_focus();
                            }
                            self.rename_edit_clicked = self.rename_edit_clicked || resp.has_focus();
                            let commit = resp.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let cancel = ui.input(|i| i.key_pressed(egui::Key::Escape));
                            if commit {
                                if let Some(parent) = entry.path.parent() {
                                    let new_name = self.rename_new_name.trim();
                                    if !new_name.is_empty() {
                                        let new_path = parent.join(new_name);
                                        if fs::rename(&entry.path, &new_path).is_ok() {
                                            self.refresh();
                                            self.select_single_item(new_path);
                                        }
                                    }
                                }
                                self.rename_target = None;
                            }
                            if cancel {
                                self.rename_target = None;
                            }
                        } else {
                            ui.label(RichText::new(&entry.name).strong());
                        }

                        if let Some(badge) = badge_opt {
                            ui.add_space(8.0);
                            ui.label(RichText::new(badge).size(11.0).color(accent_color));
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if entry.is_dir {
                                ui.weak("Folder");
                            } else if let Ok(meta) = fs::metadata(&entry.path) {
                                ui.weak(format!("{} B", meta.len()));
                            }
                        });
                    });
                });

            let response = frame_resp.response.interact(Sense::click_and_drag());
            self.handle_item_interaction(ui, entry, &response);
        }
    }

    fn draw_bottom_status_bar(&mut self, ui: &mut egui::Ui, visible_count: usize) {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(egui::Margin::symmetric(6, 3))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.weak(format!(
                        "{} items ({} selected)",
                        visible_count,
                        self.selected_paths.len()
                    ));

                    if let Some((msg, is_err)) = &self.status_message {
                        ui.separator();
                        let color = if *is_err {
                            ui.visuals().error_fg_color
                        } else {
                            COLOR_SUCCESS
                        };
                        ui.add(
                            egui::Label::new(RichText::new(msg).size(11.5).color(color))
                                .truncate(),
                        );
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.spacing_mut().slider_width = 70.0;
                        let slider = egui::Slider::new(&mut self.tile_size, 52.0..=140.0)
                            .show_value(false);
                        if ui
                            .add(slider)
                            .on_hover_text("Adjust icon size")
                            .changed()
                        {
                            if self.tile_size <= 54.0 {
                                self.view_mode = ViewMode::List;
                            } else {
                                self.view_mode = ViewMode::Grid;
                            }
                        }
                        ui.weak("");
                    });
                });
            });
    }

    fn draw_modals(&mut self, ctx: &egui::Context) {
        if let Some(target_dir) = self.show_new_folder_modal.clone() {
            egui::Window::new(" Create New Folder")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Folder Name:");
                    ui.text_edit_singleline(&mut self.new_folder_name);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let enter = ui.input(|i| i.key_pressed(Key::Enter));
                        if (ui.button(" Create").clicked() || enter)
                            && !self.new_folder_name.trim().is_empty()
                        {
                            let new_path = target_dir.join(self.new_folder_name.trim());
                            match fs::create_dir_all(&new_path) {
                                Ok(_) => {
                                    self.refresh();
                                    self.select_single_item(new_path);
                                }
                                Err(e) => {
                                    self.status_message =
                                        Some((format!("Failed to create folder: {}", e), true));
                                }
                            }
                            self.show_new_folder_modal = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_new_folder_modal = None;
                        }
                    });
                });
        }

        if let Some((target_dir, ext, display_name)) = self.create_asset_modal.clone() {
            egui::Window::new(format!(" Create {}", display_name))
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!("File name (.{}):", ext));
                    ui.text_edit_singleline(&mut self.new_asset_name);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let enter = ui.input(|i| i.key_pressed(Key::Enter));
                        if (ui.button(" Create").clicked() || enter)
                            && !self.new_asset_name.trim().is_empty()
                        {
                            let stem = self
                                .new_asset_name
                                .trim()
                                .trim_end_matches(&format!(".{}", ext))
                                .to_string();
                            let stem_ref = stem.as_str();
                            let mut created_res = None;
                            if let Ok(reg) = global_asset_registry().lock() {
                                if let Some(desc) = reg.get_by_extension(&ext) {
                                    if let Some(create_fn) = &desc.on_create {
                                        created_res = Some((create_fn)(&target_dir, stem_ref));
                                    }
                                }
                            }
                            if let Some(res) = created_res {
                                match res {
                                    Ok(created_path) => {
                                        self.refresh();
                                        self.select_single_item(created_path);
                                        self.status_message = Some((
                                            format!("Created {} (.{})", stem, ext),
                                            false,
                                        ));
                                    }
                                    Err(err) => {
                                        self.status_message = Some((err, true));
                                    }
                                }
                            }
                            self.create_asset_modal = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.create_asset_modal = None;
                        }
                    });
                });
        }

        if !self.delete_files_confirm.is_empty() {
            let to_delete = self.delete_files_confirm.clone();
            egui::Window::new(" Confirm Delete")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to permanently delete {} item(s)?",
                        to_delete.len()
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let del_btn = egui::Button::new(
                            RichText::new(" Delete").color(ui.visuals().error_fg_color),
                        );
                        if ui.add(del_btn).clicked() {
                            for p in &to_delete {
                                if self.can_modify_item(p) {
                                    if p.is_dir() {
                                        let _ = fs::remove_dir_all(p);
                                    } else {
                                        let _ = fs::remove_file(p);
                                    }
                                }
                            }
                            self.delete_files_confirm.clear();
                            self.selected_paths.clear();
                            self.refresh();
                        }
                        if ui.button("Cancel").clicked() {
                            self.delete_files_confirm.clear();
                        }
                    });
                });
        }

        if let Some(pkg) = self.active_package_modal.clone() {
            let toml_value = std::fs::read_to_string(pkg.path.join("package.toml"))
                .ok()
                .and_then(|c| c.parse::<toml::Value>().ok());

            egui::Window::new(format!(" Package Details — {}", pkg.name))
                .collapsible(false)
                .resizable(true)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.set_min_width(420.0);
                    egui::ScrollArea::vertical().max_height(460.0).show(ui, |ui| {
                        egui::Grid::new("pkg_details_grid")
                            .num_columns(2)
                            .spacing([16.0, 6.0])
                            .show(ui, |ui| {
                                ui.weak("Name:");
                                ui.label(RichText::new(&pkg.name).strong());
                                ui.end_row();

                                ui.weak("Version:");
                                ui.label(&pkg.version);
                                ui.end_row();

                                ui.weak("Category:");
                                ui.label(&pkg.category);
                                ui.end_row();

                                ui.weak("Vendor:");
                                ui.label(&pkg.vendor);
                                ui.end_row();

                                let magic = toml_value.as_ref().and_then(|v| v.get("magic").and_then(|x| x.as_str())).unwrap_or("");
                                ui.weak("Magic:");
                                ui.label(magic);
                                ui.end_row();

                                let channel = toml_value.as_ref().and_then(|v| v.get("channel").and_then(|x| x.as_str())).unwrap_or("");
                                ui.weak("Channel:");
                                ui.label(channel);
                                ui.end_row();

                                ui.weak("Status:");
                                ui.label(if pkg.enabled { "Enabled" } else { "Disabled" });
                                ui.end_row();

                                let targets = toml_value.as_ref().and_then(|v| v.get("targets").and_then(|x| x.as_array())).map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default();
                                ui.weak("Targets:");
                                ui.label(targets);
                                ui.end_row();

                                let deps = toml_value.as_ref().and_then(|v| v.get("dependencies").and_then(|x| x.as_table())).map(|t| t.keys().cloned().collect::<Vec<_>>().join(", ")).unwrap_or_default();
                                ui.weak("Dependencies:");
                                ui.label(if deps.is_empty() { "—" } else { &deps });
                                ui.end_row();

                                ui.weak("Path:");
                                ui.label(RichText::new(pkg.path.display().to_string()).monospace().size(10.5));
                                ui.end_row();
                            });

                        let description = toml_value.as_ref().and_then(|v| v.get("description").and_then(|x| x.as_str())).unwrap_or(&pkg.description);
                        if !description.is_empty() {
                            ui.separator();
                            ui.weak("Description:");
                            ui.label(description);
                        }

                        ui.separator();
                        ui.weak("Actions:");
                        ui.horizontal_wrapped(|ui| {
                            let toggle_text = if pkg.enabled { " Disable" } else { "▶ Enable" };
                            if ui.small_button(toggle_text).clicked() {
                                if let Some(cb) = &self.set_status_cb {
                                    let id = pkg.id.clone();
                                    if let Err(err) = (cb)(id, !pkg.enabled) {
                                        self.status_message = Some((err, true));
                                    } else {
                                        self.refresh();
                                    }
                                }
                            }
                            if ui.small_button(" Delete").clicked() {
                                if let Some(cb) = &self.remove_cb {
                                    let id = pkg.id.clone();
                                    let path = pkg.path.clone();
                                    if let Err(err) = (cb)(id) {
                                        self.status_message = Some((err, true));
                                    } else {
                                        if self.current_dir.starts_with(&path) {
                                            self.current_dir = self.packages_root.clone();
                                        }
                                        self.active_package_modal = None;
                                        self.refresh();
                                    }
                                }
                            }
                            if ui.small_button(" Bump").clicked() {
                                if let Some(cb) = &self.bump_cb {
                                    let id = pkg.id.clone();
                                    if let Err(err) = (cb)(id) {
                                        self.status_message = Some((err, true));
                                    } else {
                                        self.refresh();
                                    }
                                }
                            }
                            if ui.small_button("Sync").clicked() {
                                if let Some(cb) = &self.sync_cb {
                                    if let Err(err) = (cb)() {
                                        self.status_message = Some((err, true));
                                    } else {
                                        self.refresh();
                                    }
                                }
                            }
                        });

                        ui.add_space(6.0);
                        ui.weak("Install specific version:");
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(&mut self.pkg_version_input).desired_width(120.0).hint_text("e.g. 1.2.3 or latest"));
                            if ui.small_button(" Install").clicked() && !self.pkg_version_input.trim().is_empty() {
                                if let Some(cb) = &self.install_version_cb {
                                    let id = pkg.id.clone();
                                    let ver = self.pkg_version_input.trim().to_string();
                                    if let Err(err) = (cb)(id, ver) {
                                        self.status_message = Some((err, true));
                                    } else {
                                        self.refresh();
                                    }
                                }
                            }
                        });

                        if let Some(cb) = &self.versions_cb {
                            ui.add_space(4.0);
                            if ui.small_button(" List available versions").clicked() {
                                let id = pkg.id.clone();
                                match (cb)(id) {
                                    Ok(vs) => self.pkg_versions = vs,
                                    Err(err) => self.status_message = Some((err, true)),
                                }
                            }
                            if !self.pkg_versions.is_empty() {
                                ui.add_space(4.0);
                                egui::ScrollArea::vertical().max_height(120.0).show(ui, |ui| {
                                    for v in &self.pkg_versions.clone() {
                                        ui.horizontal(|ui| {
                                            ui.label(v);
                                            if ui.small_button("Install").clicked() {
                                                if let Some(cb) = &self.install_version_cb {
                                                    let id = pkg.id.clone();
                                                    if let Err(err) = (cb)(id, v.clone()) {
                                                        self.status_message = Some((err, true));
                                                    } else {
                                                        self.refresh();
                                                    }
                                                }
                                            }
                                        });
                                    }
                                });
                            }
                        }

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button(" Open Folder").clicked() {
                                self.current_dir = pkg.path.clone();
                                self.refresh();
                                self.active_package_modal = None;
                            }
                            if ui.button("Close").clicked() {
                                self.active_package_modal = None;
                                self.pkg_versions.clear();
                            }
                        });
                    });
                });
        }

        if self.show_create_modal {
            egui::Window::new(" Create New Package")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    egui::Grid::new("create_pkg_grid")
                        .num_columns(2)
                        .spacing([12.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Category:");
                            ui.text_edit_singleline(&mut self.create_form.category);
                            ui.end_row();

                            ui.label("Vendor:");
                            ui.text_edit_singleline(&mut self.create_form.vendor);
                            ui.end_row();

                            ui.label("Package Name:");
                            ui.text_edit_singleline(&mut self.create_form.name);
                            ui.end_row();

                            ui.label("Targets:");
                            ui.horizontal(|ui| {
                                ui.checkbox(&mut self.create_form.target_editor, "Editor");
                                ui.checkbox(&mut self.create_form.target_runtime, "Runtime");
                            });
                            ui.end_row();
                        });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(" Create Package").clicked() {
                            if let Some(cb) = &self.init_cb {
                                let category = self.create_form.category.clone();
                                let vendor = self.create_form.vendor.clone();
                                let name = self.create_form.name.clone();
                                let te = self.create_form.target_editor;
                                let tr = self.create_form.target_runtime;
                                match (cb)(category, vendor, name, te, tr) {
                                    Ok(_) => {
                                        self.refresh();
                                        self.show_create_modal = false;
                                    }
                                    Err(err) => {
                                        self.status_message = Some((err, true));
                                    }
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_create_modal = false;
                        }
                    });
                });
        }

        if self.show_url_modal {
            egui::Window::new(" Install Package from URL")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label("Git or ZIP URL:");
                    ui.text_edit_singleline(&mut self.install_url);
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(" Install").clicked() && !self.install_url.trim().is_empty()
                        {
                            let url = self.install_url.trim().to_string();
                            if let Some(cb) = &self.install_url_cb {
                                match (cb)(url) {
                                    Ok(msg) => {
                                        self.status_message = Some((msg, false));
                                        self.refresh();
                                        self.show_url_modal = false;
                                    }
                                    Err(err) => {
                                        self.status_message = Some((err, true));
                                    }
                                }
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_url_modal = false;
                        }
                    });
                });
        }

        if let Some(pkg) = self.delete_confirm_package.clone() {
            egui::Window::new(" Delete Package")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.label(format!(
                        "Are you sure you want to delete package '{}'?",
                        pkg.name
                    ));
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let btn = egui::Button::new(
                            RichText::new(" Delete Package").color(ui.visuals().error_fg_color),
                        );
                        if ui.add(btn).clicked() {
                            if let Some(cb) = &self.remove_cb {
                                let id = pkg.id.clone();
                                let pkg_path = pkg.path.clone();
                                if let Err(err) = (cb)(id) {
                                    self.status_message = Some((err, true));
                                } else {
                                    if self.current_dir.starts_with(&pkg_path) {
                                        self.current_dir = self.packages_root.clone();
                                    }
                                    self.refresh();
                                }
                            }
                            self.delete_confirm_package = None;
                        }
                        if ui.button("Cancel").clicked() {
                            self.delete_confirm_package = None;
                        }
                    });
                });
        }
    }
}

impl EditorWindow for GrimoireWindow {
    fn id(&self) -> WindowId {
        "grimoire".to_string()
    }

    fn title(&self) -> &str {
        "Grimoire"
    }

    fn save_state(&self) -> Option<String> {
        Some(format!("{}|{}", self.current_dir.to_string_lossy(), self.tile_size))
    }

    fn restore_state(&mut self, state: &str) {
        let parts: Vec<&str> = state.splitn(2, '|').collect();
        let p = std::path::PathBuf::from(parts[0]);
        if p.exists() {
            self.current_dir = p;
            if parts.len() == 2 {
                if let Ok(v) = parts[1].parse::<f32>() {
                    self.tile_size = v;
                }
            }
            self.refresh();
        }
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.refresh_if_requested();
        self.rename_edit_clicked = false;
        self.rename_edit_rect = None;
        self.handle_keyboard_shortcuts(ui);

        self.draw_top_toolbar(ui);
        ui.add_space(1.0);
        self.draw_breadcrumb_bar(ui);
        ui.add_space(1.0);

        let visible = self.visible_entries();
        let avail_h = (ui.available_height() - 48.0).max(80.0);

        let body_frame = egui::Frame::group(ui.style())
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::same(6));

        let body_resp = body_frame.show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(avail_h)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    if visible.is_empty() {
                        ui.add_space(36.0);
                        ui.vertical_centered(|ui| {
                            ui.weak(" This folder is empty.");
                            if self.can_write_into_dir(&self.current_dir) {
                                ui.weak(
                                    "Right-click to create folders/assets, or drag & drop files from your OS here.",
                                );
                            }
                        });
                    } else {
                        match self.view_mode {
                            ViewMode::Grid => self.draw_grid_view(ui, &visible),
                            ViewMode::List => self.draw_list_view(ui, &visible),
                        }
                    }

                    let remaining = ui.available_size().max(Vec2::new(ui.available_width(), 48.0));
                    let (_, empty_resp) = ui.allocate_exact_size(remaining, Sense::click());
                    if empty_resp.clicked() {
                        self.selected_paths.clear();
                        self.last_selected = None;
                        crate::ui::inspector::clear_inspector_target();
                        if self.rename_target.is_some() {
                            let target = self.rename_target.clone().unwrap();
                            let new_name = self.rename_new_name.trim().to_string();
                            if !new_name.is_empty() {
                                if let Some(parent) = target.parent() {
                                    let new_path = parent.join(&new_name);
                                    if fs::rename(&target, &new_path).is_ok() {
                                        self.refresh();
                                        self.select_single_item(new_path);
                                    }
                                }
                            }
                            self.rename_target = None;
                        }
                    }
                    empty_resp.context_menu(|ui| {
                        self.context_target = None;
                        self.populate_context_menu(ui, None);
                    });
                });
        });

        if !self.dragging_paths.is_empty() {
            if let Some(pos) = ui.ctx().pointer_interact_pos() {
                egui::Area::new(egui::Id::new("grimoire_drag_preview"))
                    .fixed_pos(pos + Vec2::new(14.0, 14.0))
                    .interactable(false)
                    .show(ui.ctx(), |ui| {
                        egui::Frame::popup(ui.style()).show(ui, |ui| {
                            ui.label(
                                RichText::new(format!(
                                    " Moving {} item(s)...",
                                    self.dragging_paths.len()
                                ))
                                .strong(),
                            );
                        });
                    });
            }
            if ui.input(|i| i.pointer.any_released()) {
                self.dragging_paths.clear();
            }
        }

        body_resp.response.context_menu(|ui| {
            self.context_target = None;
            self.populate_context_menu(ui, None);
        });
        self.handle_os_dropped_files(ui);

        if self.rename_target.is_some()
            && ui.ctx().input(|i| i.pointer.any_pressed())
        {
            let outside = ui.ctx().input(|i| {
                if !i.pointer.any_pressed() {
                    return false;
                }
                match i.pointer.interact_pos() {
                    Some(pos) => self.rename_edit_rect.map(|r| !r.contains(pos)).unwrap_or(true),
                    None => true,
                }
            });
            if outside {
                let target = self.rename_target.clone().unwrap();
                let new_name = self.rename_new_name.trim().to_string();
                if !new_name.is_empty() {
                    if let Some(parent) = target.parent() {
                        let new_path = parent.join(&new_name);
                        if fs::rename(&target, &new_path).is_ok() {
                            self.refresh();
                            self.select_single_item(new_path);
                        }
                    }
                }
                self.rename_target = None;
            }
        }

        ui.add_space(4.0);
        self.draw_bottom_status_bar(ui, visible.len());

        let ctx = ui.ctx().clone();
        self.draw_modals(&ctx);
    }
}