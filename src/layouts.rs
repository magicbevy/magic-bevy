use std::path::Path;

use magic_core::*;

pub struct LayoutManager {}

impl LayoutManager {
    pub fn new() -> Self {
        Self {}
    }

    #[allow(dead_code)]
    pub fn save_layout(&self, windows: &mut WindowInstances, name: &str) {
        let layouts_dir = Path::new("./layouts");
        let _ = std::fs::create_dir_all(layouts_dir);

        let layout_path = layouts_dir.join(format!("{}.toml", name));

        let mut content = format!("name = \"{}\"\n\n[[tabs]]\n", name);
        for instance_id in windows.iter_ids().collect::<Vec<_>>() {
            if let Some(instance) = windows.get_mut(instance_id) {
                content.push_str(&format!(
                    "window_id = \"{}\"\ninstance_id = {}\ntitle = \"{}\"\nopen = {}\n\n",
                    instance.window_id,
                    instance_id,
                    instance.title,
                    instance.open
                ));
            }
        }

        let _ = std::fs::write(&layout_path, content);
        mgsuccess!("Layout", "Layout '{}' saved to {}", name, layout_path.display());
    }

    #[allow(dead_code)]
    pub fn load_layout(&self, windows: &mut WindowInstances, window_registry: &WindowRegistry, name: &str) {
        let layouts_dir = Path::new("./layouts");
        let layout_path = layouts_dir.join(format!("{}.toml", name));

        if !layout_path.exists() {
            mgwarn!("Layout", "Layout '{}' not found at {}", name, layout_path.display());
            return;
        }

        // Close all existing windows
        for instance_id in windows.iter_ids().collect::<Vec<_>>() {
            windows.remove(instance_id);
        }

        let content = match std::fs::read_to_string(&layout_path) {
            Ok(c) => c,
            Err(e) => {
                mgwarn!("Layout", "Failed to read layout '{}': {}", name, e);
                return;
            }
        };

        let mut current_window_id = String::new();
        let mut _current_title = String::new();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("window_id = ") {
                current_window_id = line
                    .strip_prefix("window_id = \"")
                    .unwrap_or("")
                    .strip_suffix('"')
                    .unwrap_or("")
                    .to_string();
            } else if line.starts_with("title = ") {
                _current_title = line
                    .strip_prefix("title = \"")
                    .unwrap_or("")
                    .strip_suffix('"')
                    .unwrap_or("")
                    .to_string();

                if !current_window_id.is_empty() {
                    if let Some(factory) = window_registry.factories.get(&current_window_id) {
                        let window = (factory.create)();
                        windows.add(current_window_id.clone(), _current_title.clone(), window);
                    }
                }

                current_window_id.clear();
                _current_title.clear();
            }
        }

        mgsuccess!("Layout", "Layout '{}' loaded", name);
    }

    pub fn list_layouts(&self) -> Vec<String> {
        let layouts_dir = Path::new("./layouts");
        if !layouts_dir.exists() {
            return Vec::new();
        }

        let mut layouts = Vec::new();
        if let Ok(entries) = std::fs::read_dir(layouts_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".toml") {
                        layouts.push(name.strip_suffix(".toml").unwrap_or(name).to_string());
                    }
                }
            }
        }
        layouts
    }
}
