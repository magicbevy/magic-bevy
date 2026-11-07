use magic_api::RuntimeRegistry;

#[unsafe(no_mangle)]
pub extern "Rust" fn register_all_packages(registry: &mut RuntimeRegistry) {
    if let Some(cb) = registry.log_fn {
        magic_api::log::set_log_callback(cb);
    }
    magic_pkg_runtime::register_all_packages(registry);
}

pub fn run() {
    // Standalone game entry point. Package registration happens here.
    let mut registry = RuntimeRegistry::new();
    register_all_packages(&mut registry);
    // TODO: init window/render/event loop and execute runtime packages.
    println!("magic_runtime: {} packages registered", registry.registered);
}
