pub mod events;
pub mod menu;
pub mod runtime;

#[cfg(feature = "ui")]
pub mod assets;
#[cfg(feature = "ui")]
pub mod window;
#[cfg(feature = "ui")]
pub mod editor;

pub use events::EditorEventRegistry;
pub use menu::{MenuItem, MenuRegistry};
pub use runtime::RuntimeRegistry;

#[cfg(feature = "ui")]
pub use assets::{AssetTypeDescriptor, AssetTypeRegistry, global_asset_registry};
#[cfg(feature = "ui")]
pub use editor::EditorRegistry;
#[cfg(feature = "ui")]
pub use window::{EditorWindow, WindowDescriptor, WindowFactory, WindowId, WindowRegistry, WindowInstance, WindowInstances, WindowTab};
