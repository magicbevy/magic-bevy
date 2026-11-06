pub use magic_api::*;

pub mod ui;
pub mod package_manager;

pub use ui::top_bar::{draw_top_bar, LayoutAction};
pub use ui::oracle_logs::OracleLogsWindow;
pub use ui::grimoire::window::GrimoireWindow;
pub use ui::grimoire::types::PackageInfo;
pub use ui::inspector::InspectorWindow;
pub use ui::inspector::{InspectorTarget, get_inspector_target, register_inspector, set_inspector_target, set_inspector_target_file, unregister_inspector};
pub use ui::grimoire::window::request_grimoire_refresh;

pub use eframe::egui;
