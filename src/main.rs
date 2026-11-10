mod app;
mod engine_version;
mod hot_reload;
mod windows;
mod layouts;

use magic_api::mgwarn;
use magic_core::*;
use std::env;
use std::panic;

fn main() -> Result<(), eframe::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1] == "bump" {
        return match engine_version::bump() {
            Ok(()) => Ok(()),
            Err(e) => {
                mgwarn!(engine_version::TAG, format!("engine bump failed: {e}"));
                Ok(())
            }
        };
    }

    panic::set_hook(Box::new(|info| {
        let payload = if let Some(s) = info.payload().downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic".to_string()
        };
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();
        let entry = magic_core::log::LogEntry::new(
            magic_core::log::LogLevel::Error,
            format!("Panic at {}: {}", location, payload),
            "panic_hook",
        );
        if let Ok(mut store) = magic_core::log::global_store().lock() {
            store.push(entry);
        }
    }));

    std::fs::create_dir_all("./packages").unwrap();
    std::fs::create_dir_all("./layouts").unwrap();
    std::fs::create_dir_all("./.magic").unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("MagicBevy")
            .with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };

    eframe::run_native(
        "magic-bevy",
        options,
        Box::new(|cc| Ok(Box::new(app::MagicBevyApp::new(cc)))),
    )
}
