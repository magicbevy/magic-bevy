pub mod editor;
pub mod events;
pub mod menu;
pub mod runtime;
pub mod window;

pub use editor::EditorRegistry;
pub use events::EditorEventRegistry;
pub use menu::{MenuItem, MenuRegistry};
pub use runtime::RuntimeRegistry;
pub use window::{EditorWindow, WindowDescriptor, WindowFactory, WindowId, WindowRegistry, WindowInstance, WindowInstances};