use crate::api::assets::AssetTypeRegistry;
use crate::api::events::EditorEventRegistry;
use crate::api::menu::MenuRegistry;
use crate::api::window::WindowRegistry;
use std::sync::mpsc::Sender;

pub struct EditorRegistry<'a> {
    pub menu: &'a mut MenuRegistry,
    pub events: &'a mut EditorEventRegistry,
    pub windows: &'a mut WindowRegistry,
    pub assets: &'a mut AssetTypeRegistry,
    pub open_window_tx: Sender<crate::api::window::WindowId>,
    pub log_fn: Option<fn(magic_log::LogLevel, &str, &str)>,
}
