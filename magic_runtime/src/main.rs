use magic_api_runtime::api::runtime::RenderContext;
use magic_api_runtime::{RuntimeRegistry, UpdateRegistry};
use magic_log::mgsuccess;
use std::sync::{Arc, Mutex};

fn main() {
    let mut updates = UpdateRegistry::new(60.0, 1.0);
    let render_context = Arc::new(Mutex::new(RenderContext::default()));

    let mut registry = RuntimeRegistry {
        render_context: render_context.clone(),
        log_fn: None,
        updates: &mut updates,
    };

    magic_runtime::register_all_packages(&mut registry);
    mgsuccess!("Runtime", "All packages registered successfully");

    updates.add_update(move |_dt| {

    });

    updates.run();
}