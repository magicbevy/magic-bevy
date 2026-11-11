pub mod profile;
pub mod systems;

pub use systems::{EditorSystem, SystemManager};

use magic_api_editor::EditorRegistry;

#[unsafe(no_mangle)]
pub extern "Rust" fn register_all_packages(registry: &mut EditorRegistry) {
    if let Some(cb) = registry.log_fn {
        magic_log::set_log_callback(cb);
    }
    magic_pkg_editor::register_all_packages(registry);
}
