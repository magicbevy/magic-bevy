#![allow(dead_code)]

use eframe::egui;

pub struct ContextMenuItem {
    pub label: String,
    pub icon: Option<String>,
    pub enabled: bool,
    pub action: Box<dyn Fn() + Send + Sync>,
}

impl ContextMenuItem {
    pub fn new(label: impl Into<String>, action: Box<dyn Fn() + Send + Sync>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            enabled: true,
            action,
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }
}

#[derive(Default)]
pub struct ContextMenuRegistry {
    items: Vec<ContextMenuItem>,
}

impl ContextMenuRegistry {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: ContextMenuItem) {
        self.items.push(item);
    }

    pub fn clear(&mut self) {
        self.items.clear();
    }

    pub fn items(&self) -> &[ContextMenuItem] {
        &self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub fn draw_empty_context_menu(ui: &mut egui::Ui) {
    let _ = ui;
}
