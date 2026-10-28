use egui_dock::DockState;
use magic_core::*;
use serde_json;

fn magic_bevy_dir() -> std::path::PathBuf {
    std::env::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".magic-bevy")
}

fn project_layouts_dir() -> std::path::PathBuf {
    std::path::PathBuf::from("./layouts")
}

fn global_layouts_dir() -> std::path::PathBuf {
    magic_bevy_dir().join("layouts")
}

pub const TEMP_LAYOUT_NAME: &str = ".temp.layout";

pub struct LayoutManager {
    active_layout: Option<String>,
    custom_layout_name: String,
}

impl LayoutManager {
    pub fn new() -> Self {
        Self {
            active_layout: None,
            custom_layout_name: "custom".to_string(),
        }
    }

    pub fn set_active_layout(&mut self, name: String) {
        self.active_layout = Some(name.clone());
        self.save_active_layout_state(&name);
    }

    pub fn get_active_layout(&self) -> Option<&str> {
        self.active_layout.as_deref()
    }

    #[allow(dead_code)]
    pub fn get_custom_layout_name(&self) -> &str {
        &self.custom_layout_name
    }

    fn save_active_layout_state(&self, name: &str) {
        let layouts_dir = project_layouts_dir();
        let _ = std::fs::create_dir_all(&layouts_dir);
        let path = layouts_dir.join("active_layout.json");
        let content = serde_json::to_string(&serde_json::json!({ "active_layout": name }))
            .unwrap_or_else(|e| {
                magic_core::oracle_error!("Layout", "Failed to serialize active layout: {}", e);
                String::new()
            });
        let _ = std::fs::write(&path, content);
    }

    #[allow(dead_code)]
    fn load_active_layout_state(&mut self) {
        let path = project_layouts_dir().join("active_layout.json");
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(name) = json.get("active_layout").and_then(|v| v.as_str()) {
                        self.active_layout = Some(name.to_string());
                    }
                }
            }
        }
    }

    pub fn save_temp_layout(&self, dock_state: &DockState<WindowTab>) {
        let layouts_dir = project_layouts_dir();
        let _ = std::fs::create_dir_all(&layouts_dir);
        let layout_path = layouts_dir.join(".temp.layout");
        let json = serde_json::to_string_pretty(dock_state)
            .unwrap_or_else(|e| {
                magic_core::oracle_error!("Layout", "Failed to serialize temp layout: {}", e);
                String::new()
            });
        let _ = std::fs::write(&layout_path, json);
    }

    pub fn load_temp_layout(&self, dock_state: &mut DockState<WindowTab>) -> bool {
        let layout_path = project_layouts_dir().join(".temp.layout");
        if !layout_path.exists() {
            return false;
        }
        let json = match std::fs::read_to_string(&layout_path) {
            Ok(c) => c,
            Err(_) => return false,
        };
        match serde_json::from_str::<DockState<WindowTab>>(&json) {
            Ok(state) => {
                *dock_state = state;
                true
            }
            Err(_) => false,
        }
    }

    #[allow(dead_code)]
    pub fn clear_temp_layout(&self) {
        let layout_path = project_layouts_dir().join(".temp.layout");
        let _ = std::fs::remove_file(&layout_path);
    }

    #[allow(dead_code)]
    pub fn save_layout(&self, dock_state: &DockState<WindowTab>, name: &str, is_global: bool) {
        let layouts_dir = if is_global {
            global_layouts_dir()
        } else {
            project_layouts_dir()
        };
        let _ = std::fs::create_dir_all(&layouts_dir);

        let layout_path = layouts_dir.join(format!("{}.layout", name));

        let json = serde_json::to_string_pretty(dock_state)
            .unwrap_or_else(|e| {
                magic_core::oracle_error!("Layout", "Failed to serialize layout '{}': {}", name, e);
                String::new()
            });
        let _ = std::fs::write(&layout_path, json);
        mgsuccess!("Layout", "Layout '{}' saved to {}", name, layout_path.display());
    }

    #[allow(dead_code)]
    pub fn load_layout(&self, dock_state: &mut DockState<WindowTab>, name: &str, is_global: bool) {
        let layouts_dir = if is_global {
            global_layouts_dir()
        } else {
            project_layouts_dir()
        };
        let layout_path = layouts_dir.join(format!("{}.layout", name));

        if !layout_path.exists() {
            mgwarn!("Layout", "Layout '{}' not found at {}", name, layout_path.display());
            return;
        }

        let json = match std::fs::read_to_string(&layout_path) {
            Ok(c) => c,
            Err(e) => {
                mgwarn!("Layout", "Failed to read layout '{}': {}", name, e);
                return;
            }
        };

        match serde_json::from_str::<DockState<WindowTab>>(&json) {
            Ok(state) => {
                *dock_state = state;
                mgsuccess!("Layout", "Layout '{}' loaded", name);
            }
            Err(e) => mgwarn!("Layout", "Failed to load layout '{}': {}", name, e),
        }
    }

    pub fn list_project_layouts(&self) -> Vec<String> {
        Self::list_layouts_in_dir(&project_layouts_dir())
    }

    pub fn list_global_layouts(&self) -> Vec<String> {
        Self::list_layouts_in_dir(&global_layouts_dir())
    }

    fn list_layouts_in_dir(dir: &std::path::Path) -> Vec<String> {
        if !dir.exists() {
            return Vec::new();
        }

        let mut layouts = Vec::new();
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name == TEMP_LAYOUT_NAME {
                        continue;
                    }
                    if name.ends_with(".layout") {
                        layouts.push(name.strip_suffix(".layout").unwrap_or(name).to_string());
                    }
                }
            }
        }
        layouts.sort();
        layouts
    }

    pub fn delete_layout(&self, name: &str, is_global: bool) {
        if !is_global {
            mgwarn!("Layout", "Cannot delete project layout '{}'", name);
            return;
        }

        let layout_path = global_layouts_dir().join(format!("{}.layout", name));

        if layout_path.exists() {
            if let Err(e) = std::fs::remove_file(&layout_path) {
                mgwarn!("Layout", "Failed to delete layout '{}': {}", name, e);
            } else {
                mgsuccess!("Layout", "Layout '{}' deleted", name);
            }
        } else {
            mgwarn!("Layout", "Layout '{}' not found", name);
        }
    }

    pub fn is_project_layout(&self, name: &str) -> bool {
        project_layouts_dir().join(format!("{}.layout", name)).exists()
    }

    pub fn is_global_layout(&self, name: &str) -> bool {
        global_layouts_dir().join(format!("{}.layout", name)).exists()
    }
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}
