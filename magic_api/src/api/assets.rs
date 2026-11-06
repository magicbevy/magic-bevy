use eframe::egui::{self, Color32};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

pub type CreateAssetFn =
    Box<dyn Fn(&Path, &str) -> Result<PathBuf, String> + Send + Sync + 'static>;
pub type OpenAssetFn = Box<dyn Fn(&Path) + Send + Sync + 'static>;
pub type InspectAssetFn = Box<dyn FnMut(&mut egui::Ui, &[PathBuf]) + Send + 'static>;

pub struct AssetTypeDescriptor {
    pub extension: String,
    pub display_name: String,
    pub category: Option<String>,
    pub package_name: Option<String>,
    pub icon: String,
    pub color: Color32,
    pub default_name: Option<String>,
    pub folders: Vec<String>,
    pub subfolders: Vec<String>,
    pub on_create: Option<CreateAssetFn>,
    pub on_open: Option<OpenAssetFn>,
    pub on_inspect: Option<InspectAssetFn>,
}

impl AssetTypeDescriptor {
    pub fn new(
        extension: impl Into<String>,
        display_name: impl Into<String>,
        icon: impl Into<String>,
    ) -> Self {
        let ext = extension
            .into()
            .trim_start_matches('.')
            .to_lowercase();
        Self {
            extension: ext,
            display_name: display_name.into(),
            category: None,
            package_name: None,
            icon: icon.into(),
            color: Color32::from_rgb(110, 86, 240),
            default_name: None,
            folders: vec!["assets".to_string(), "editor".to_string(), "runtime".to_string()],
            subfolders: Vec::new(),
            on_create: None,
            on_open: None,
            on_inspect: None,
        }
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn package(mut self, package_name: impl Into<String>) -> Self {
        self.package_name = Some(package_name.into());
        self
    }

    pub fn color(mut self, color: Color32) -> Self {
        self.color = color;
        self
    }

    pub fn default_name(mut self, name: impl Into<String>) -> Self {
        self.default_name = Some(name.into());
        self
    }

    pub fn folders<I, S>(mut self, folders: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.folders = folders.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn subfolders<I, S>(mut self, subfolders: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.subfolders = subfolders.into_iter().map(|s| s.into()).collect();
        self
    }

    pub fn allowed_in(&self, section: &str) -> bool {
        self.folders.is_empty() || self.folders.iter().any(|f| f == section)
    }

    pub fn allowed_in_subfolder(&self, subdir: Option<&str>) -> bool {
        if self.subfolders.is_empty() {
            return true;
        }
        match subdir {
            Some(s) => self.subfolders.iter().any(|f| f == s),
            None => false,
        }
    }

    pub fn on_create<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Path, &str) -> Result<PathBuf, String> + Send + Sync + 'static,
    {
        self.on_create = Some(Box::new(callback));
        self
    }

    pub fn on_open<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Path) + Send + Sync + 'static,
    {
        self.on_open = Some(Box::new(callback));
        self
    }

    pub fn on_inspect<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&mut egui::Ui, &Path) + Send + 'static,
    {
        // adapt single-path callback to slice API
        let mut cb = callback;
        self.on_inspect = Some(Box::new(move |ui, paths| {
            if let Some(p) = paths.first() {
                cb(ui, p);
            }
        }));
        self
    }

    pub fn on_inspect_many<F>(mut self, callback: F) -> Self
    where
        F: FnMut(&mut egui::Ui, &[PathBuf]) + Send + 'static,
    {
        self.on_inspect = Some(Box::new(callback));
        self
    }
}

#[derive(Default)]
pub struct AssetTypeRegistry {
    types: HashMap<String, AssetTypeDescriptor>,
    selected_path: Option<PathBuf>,
}

impl AssetTypeRegistry {
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            selected_path: None,
        }
    }

    pub fn register(&mut self, descriptor: AssetTypeDescriptor) {
        self.types.insert(descriptor.extension.clone(), descriptor);
    }

    pub fn get_by_extension(&self, ext: &str) -> Option<&AssetTypeDescriptor> {
        let clean = ext.trim_start_matches('.').to_lowercase();
        self.types.get(&clean)
    }

    pub fn get_by_path(&self, path: &Path) -> Option<&AssetTypeDescriptor> {
        let ext = path.extension()?.to_str()?;
        self.get_by_extension(ext)
    }

    pub fn get_mut_by_path(&mut self, path: &Path) -> Option<&mut AssetTypeDescriptor> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        self.types.get_mut(&ext)
    }

    pub fn all_types(&self) -> impl Iterator<Item = &AssetTypeDescriptor> {
        self.types.values()
    }

    pub fn types_for_section(&self, section: &str) -> Vec<&AssetTypeDescriptor> {
        self.types
            .values()
            .filter(|d| d.allowed_in(section))
            .collect()
    }

    pub fn set_selected_path(&mut self, path: Option<PathBuf>) {
        self.selected_path = path;
    }

    pub fn selected_path(&self) -> Option<&PathBuf> {
        self.selected_path.as_ref()
    }

    pub fn clear_types(&mut self) {
        self.types.clear();
    }
}

static GLOBAL_ASSET_REGISTRY: OnceLock<Arc<Mutex<AssetTypeRegistry>>> = OnceLock::new();

pub fn global_asset_registry() -> &'static Arc<Mutex<AssetTypeRegistry>> {
    GLOBAL_ASSET_REGISTRY.get_or_init(|| Arc::new(Mutex::new(AssetTypeRegistry::new())))
}