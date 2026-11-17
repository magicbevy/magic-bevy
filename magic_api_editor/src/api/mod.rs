pub mod events;
pub mod menu;

pub mod assets;
pub mod editor;
pub mod window;

pub use events::EditorEventRegistry;
pub use menu::{MenuItem, MenuRegistry};

pub use assets::{AssetTypeDescriptor, AssetTypeRegistry, global_asset_registry};
pub use editor::EditorRegistry;
pub use window::{
    EditorWindow, WindowDescriptor, WindowFactory, WindowId, WindowInstance, WindowInstances,
    WindowRegistry, WindowTab,
};