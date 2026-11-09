use magic_api::{RuntimeRegistry, UpdateRegistry, mgsuccess};

fn main() {
    let mut updates = UpdateRegistry::new(60.0, 1.0);

    let mut registry = RuntimeRegistry {
        render_context: None,
        log_fn: None,
        updates: &mut updates,
    };

    magic_pkg_runtime::register_all_packages(&mut registry);

    mgsuccess!("Runtime", "All packages registered successfully");

    updates.add_update(|_dt| {

    });

    updates.run();
}