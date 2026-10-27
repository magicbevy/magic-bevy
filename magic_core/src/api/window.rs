use std::hash::Hash;

pub type WindowId = String;

pub trait EditorWindow: Send + Sync {
    fn id(&self) -> WindowId;
    fn title(&self) -> &str;
    fn draw(&mut self, ui: &mut egui::Ui);
}

pub struct WindowDescriptor {
    pub id: WindowId,
    pub label: String,
    pub default_width: f32,
    pub default_height: f32,
    pub dockable: bool,
}

pub struct WindowFactory {
    pub create: Box<dyn Fn() -> Box<dyn EditorWindow> + Send + Sync>,
}

pub struct WindowRegistry {
    pub descriptors: Vec<WindowDescriptor>,
    pub factories: std::collections::HashMap<WindowId, WindowFactory>,
}

impl WindowRegistry {
    pub fn new() -> Self {
        Self {
            descriptors: Vec::new(),
            factories: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, descriptor: WindowDescriptor, factory: WindowFactory) {
        self.factories.insert(descriptor.id.clone(), factory);
        self.descriptors.push(descriptor);
    }

    pub fn find_descriptor(&self, window_id: &str) -> Option<&WindowDescriptor> {
        self.descriptors.iter().find(|d| d.id == window_id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct WindowTab {
    pub window_id: WindowId,
    pub instance_id: u64,
}

pub struct WindowInstance {
    pub id: u64,
    pub window_id: WindowId,
    pub title: String,
    pub window: Box<dyn EditorWindow>,
    pub open: bool,
    pub rect: Option<egui::Rect>,
}

pub struct WindowInstances {
    instances: std::collections::HashMap<u64, WindowInstance>,
    next_id: u64,
}

impl WindowInstances {
    pub fn new() -> Self {
        Self {
            instances: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    pub fn add(&mut self, window_id: WindowId, title: String, window: Box<dyn EditorWindow>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.instances.insert(id, WindowInstance {
            id,
            window_id,
            title,
            window,
            open: true,
            rect: None,
        });
        id
    }

    pub fn remove(&mut self, id: u64) {
        self.instances.remove(&id);
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut WindowInstance> {
        self.instances.get_mut(&id)
    }

    pub fn get_title(&self, id: u64) -> Option<&str> {
        self.instances.get(&id).map(|w| w.title.as_str())
    }

    pub fn get_window_id(&self, id: u64) -> Option<&WindowId> {
        self.instances.get(&id).map(|w| &w.window_id)
    }

    pub fn iter_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.instances.keys().copied()
    }

    pub fn len(&self) -> usize {
        self.instances.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instances.is_empty()
    }
}

impl Default for WindowInstances {
    fn default() -> Self {
        Self::new()
    }
}
