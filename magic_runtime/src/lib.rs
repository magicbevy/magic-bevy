use magic_api::RuntimeRegistry;

#[unsafe(no_mangle)]
pub extern "Rust" fn register_all_packages(registry: &mut RuntimeRegistry) {
    
    if let Some(cb) = registry.log_fn {
        magic_api::log::set_log_callback(cb);
    }
    magic_pkg_runtime::register_all_packages(registry);
}
