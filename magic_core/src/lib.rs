pub use magic_api_editor::*;
pub use magic_log::{
    global_store, mgerror, mgprint, mgsuccess, mgwarn, oracle_debug, oracle_error, oracle_info,
    oracle_success, oracle_warn, LogEntry, LogLevel, LogStore,
};
pub use magic_log as log;

pub mod ui;
pub mod package_manager;

pub use ui::top_bar::{draw_top_bar, LayoutAction};
pub use ui::oracle_logs::OracleLogsWindow;
pub use ui::grimoire::window::GrimoireWindow;
pub use ui::grimoire::types::PackageInfo;
pub use ui::inspector::InspectorWindow;
pub use ui::inspector::{InspectorTarget, clear_inspector_target, get_inspector_target, register_inspector, set_inspector_target, set_inspector_target_file, set_inspector_target_files, unregister_inspector};
pub use ui::grimoire::window::request_grimoire_refresh;

pub use eframe::egui;
