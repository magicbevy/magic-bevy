use std::sync::mpsc::{channel, Receiver, Sender};

use crate::app::EngineTheme;
use crate::layouts::LayoutManager;
use eframe::egui;
use egui_dock::{DockState, TabViewer};
use magic_core::*;
use serde_json;

pub struct WindowManager {
    pub window_instances: WindowInstances,
    pub tx: Sender<WindowId>,
    pub rx: Receiver<WindowId>,
    pub layout_manager: LayoutManager,
    pub dock_state: DockState<WindowTab>,
    pub layout_dirty: bool,
}

impl WindowManager {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            window_instances: WindowInstances::new(),
            tx,
            rx,
            layout_manager: LayoutManager::new(),
            dock_state: DockState::new(Vec::new()),
            layout_dirty: true,
        }
    }

    pub fn open_window(&mut self, windows: &WindowRegistry, window_id: &str) {
        let factory = windows.factories.get(window_id);
        if let Some(factory) = factory {
            let window = (factory.create)();
            let title = window.title().to_string();
            let instance_id = self
                .window_instances
                .add(window_id.to_string(), title.clone(), window);
            let tab = WindowTab {
                window_id: window_id.to_string(),
                instance_id,
            };
            self.dock_state.push_to_focused_leaf(tab);
            self.layout_dirty = true;
            mgsuccess!("Window", "Opened '{}' (id: {})", title, instance_id);
        } else {
            mgwarn!(
                "Window",
                "No factory registered for window '{}'",
                window_id
            );
        }
    }

    #[allow(dead_code)]
    pub fn close_window(&mut self, tab: &WindowTab) {
        self.window_instances.remove(tab.instance_id);
        self.layout_dirty = true;
    }

    pub fn save_layout_named(&mut self, name: &str, is_global: bool) {
        self.layout_manager
            .save_layout(&self.dock_state, name, is_global);
        self.layout_manager.clear_temp_layout();
    }

    pub fn load_layout_named(&mut self, windows: &WindowRegistry, name: &str, is_global: bool) {
        self.layout_manager
            .load_layout(&mut self.dock_state, name, is_global);
        self.rebuild_instances_for_layout(windows);
        self.layout_dirty = true;
    }

    pub fn load_temp_layout(&mut self, windows: &WindowRegistry) -> bool {
        let loaded = self.layout_manager.load_temp_layout(&mut self.dock_state);
        if loaded {
            self.rebuild_instances_for_layout(windows);
            self.layout_dirty = true;
        }
        loaded
    }

    fn rebuild_instances_for_layout(&mut self, windows: &WindowRegistry) {
        let (dock_state, window_instances) = (&mut self.dock_state, &mut self.window_instances);

        *window_instances = WindowInstances::new();

        for (_, tab) in dock_state.iter_all_tabs_mut() {
            if let Some(factory) = windows.factories.get(&tab.window_id) {
                let window = (factory.create)();
                let title = window.title().to_string();
                let new_id = window_instances.add(tab.window_id.clone(), title, window);
                tab.instance_id = new_id;
            }
        }
    }

    pub fn list_project_layouts(&self) -> Vec<String> {
        self.layout_manager.list_project_layouts()
    }

    pub fn list_global_layouts(&self) -> Vec<String> {
        self.layout_manager.list_global_layouts()
    }

    pub fn delete_layout(&mut self, name: &str, is_global: bool) {
        self.layout_manager.delete_layout(name, is_global);
    }

    pub fn set_active_layout(&mut self, name: &str) {
        self.layout_manager.set_active_layout(name.to_string());
    }

    pub fn get_active_layout(&self) -> Option<&str> {
        self.layout_manager.get_active_layout()
    }


    fn custom_dock_style(egui_style: &egui::Style) -> egui_dock::Style {
        let mut style = egui_dock::Style::from_egui(egui_style);


        style.tab_bar.bg_fill = EngineTheme::BG_DEEP;
        style.tab_bar.height = 30.0;
        style.tab_bar.hline_color = EngineTheme::BORDER;


        style.separator.width = 3.0;
        style.separator.color_idle = EngineTheme::BG_DEEP;
        style.separator.color_hovered = EngineTheme::ACCENT;
        style.separator.color_dragged = EngineTheme::ACCENT_CYAN;


        style.tab.tab_body.bg_fill = EngineTheme::BG_DARK;
        style.tab.tab_body.stroke = egui::Stroke::new(1.0, EngineTheme::BORDER);
        style.tab.tab_body.inner_margin = egui::Margin::same(4);


        style.tab.active.bg_fill = EngineTheme::BG_DARK;
        style.tab.active.text_color = EngineTheme::TEXT_MAIN;
        style.tab.active.outline_color = EngineTheme::BORDER_ACTIVE;


        style.tab.focused.bg_fill = EngineTheme::ITEM_SELECTED;
        style.tab.focused.text_color = egui::Color32::WHITE;
        style.tab.focused.outline_color = EngineTheme::ACCENT_CYAN;


        style.tab.inactive.bg_fill = EngineTheme::BG_DEEP;
        style.tab.inactive.text_color = EngineTheme::TEXT_MUTED;
        style.tab.inactive.outline_color = egui::Color32::TRANSPARENT;


        style.tab.hovered.bg_fill = EngineTheme::ITEM_HOVER;
        style.tab.hovered.text_color = egui::Color32::WHITE;
        style.tab.hovered.outline_color = EngineTheme::BORDER_ACTIVE;

        style
    }

    pub fn render_windows(&mut self, ui: &mut egui::Ui, _windows: &mut WindowRegistry) {
        let mut tab_viewer = WindowTabViewer {
            instances: &mut self.window_instances,
        };

        let old_json = serde_json::to_string(&self.dock_state).unwrap_or_default();

        let dock_style = Self::custom_dock_style(ui.style().as_ref());

        egui_dock::DockArea::new(&mut self.dock_state)
            .style(dock_style)
            .show_inside(ui, &mut tab_viewer);

        let new_json = serde_json::to_string(&self.dock_state).unwrap_or_default();
        if old_json != new_json {
            self.layout_dirty = true;
        }


        let open_instance_ids: Vec<u64> = self
            .dock_state
            .iter_all_tabs()
            .map(|(_, tab)| tab.instance_id)
            .collect();

        let mut to_remove: Vec<u64> = Vec::new();
        for instance_id in self.window_instances.iter_ids().collect::<Vec<_>>() {
            if !open_instance_ids.contains(&instance_id) {
                to_remove.push(instance_id);
            }
        }
        for id in to_remove {
            self.window_instances.remove(id);
        }


        if self.layout_dirty {
            self.layout_manager.save_temp_layout(&self.dock_state);
            self.layout_dirty = false;
        }
    }
}

struct WindowTabViewer<'a> {
    instances: &'a mut WindowInstances,
}

impl<'a> TabViewer for WindowTabViewer<'a> {
    type Tab = WindowTab;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut WindowTab) {

        ui.set_min_height(120.0);
        ui.set_min_width(200.0);

        if let Some(instance) = self.instances.get_mut(tab.instance_id) {
            instance.window.draw(ui);
        }
    }

    fn title(&mut self, tab: &mut WindowTab) -> egui::WidgetText {
        let raw_title = if let Some(title) = self.instances.get_title(tab.instance_id) {
            title.to_string()
        } else {
            tab.window_id.clone()
        };

        let decorated = match tab.window_id.as_str() {
            "grimoire" => raw_title.to_string(),
            "oracle_logs" => raw_title.to_string(),
            _ => raw_title.to_string(),
        };

        egui::RichText::new(decorated).strong().size(12.5).into()
    }

    fn closeable(&mut self, _tab: &mut WindowTab) -> bool {
        true
    }

    fn id(&mut self, tab: &mut WindowTab) -> egui::Id {
        egui::Id::new(format!("{}:{}", tab.window_id, tab.instance_id))
    }
}