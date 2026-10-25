use std::sync::mpsc::{Receiver, Sender, channel};

use crate::layouts::LayoutManager;
use magic_core::*;

pub struct WindowManager {
    pub window_instances: WindowInstances,
    pub tx: Sender<WindowId>,
    pub rx: Receiver<WindowId>,
    pub layout_manager: LayoutManager,
}

impl WindowManager {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            window_instances: WindowInstances::new(),
            tx,
            rx,
            layout_manager: LayoutManager::new(),
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
            mgsuccess!("Window", "Opened '{}' (id: {})", title, instance_id);
        } else {
            mgwarn!("Window", "No factory registered for window '{}'", window_id);
        }
    }

    pub fn render_windows(&mut self, ctx: &eframe::egui::Context, _windows: &mut WindowRegistry) {
        let mut windows_to_close: Vec<u64> = Vec::new();
        for instance_id in self.window_instances.iter_ids().collect::<Vec<_>>() {
            if let Some(instance) = self.window_instances.get_mut(instance_id) {
                if !instance.open {
                    windows_to_close.push(instance_id);
                    continue;
                }

                let title = instance.title.clone();
                let is_open = &mut instance.open;

                eframe::egui::Window::new(&title)
                    .id(crate::app::source(instance_id))
                    .open(is_open)
                    .resizable(true)
                    .collapsible(true)
                    .default_size([400.0, 300.0])
                    .show(ctx, |ui| {
                        instance.window.draw(ui);
                    });
            }
        }

        // Close windows that were closed by user
        for id in windows_to_close {
            self.window_instances.remove(id);
        }
    }
}
