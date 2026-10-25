mod app;
mod hot_reload;
mod windows;
mod layouts;

use magic_core::*;

fn main() -> Result<(), eframe::Error> {
    std::fs::create_dir_all("./packages").unwrap();
    std::fs::create_dir_all("./layouts").unwrap();

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
