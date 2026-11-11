pub mod events;
pub mod menu;

#[cfg(feature = "ui")]
pub mod assets;
#[cfg(feature = "ui")]
pub mod editor;
#[cfg(feature = "ui")]
pub mod window;

pub use events::EditorEventRegistry;
pub use menu::{MenuItem, MenuRegistry};

#[cfg(feature = "ui")]
pub use assets::{AssetTypeDescriptor, AssetTypeRegistry, global_asset_registry};
#[cfg(feature = "ui")]
pub use editor::EditorRegistry;
#[cfg(feature = "ui")]
pub use window::{
    EditorWindow, WindowDescriptor, WindowFactory, WindowId, WindowInstance, WindowInstances,
    WindowRegistry, WindowTab,
};
