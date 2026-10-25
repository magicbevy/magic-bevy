use eframe::egui;
use crate::api::{MenuItem, MenuRegistry};
use crate::status::CompileStatus;
use crate::ui::indicator::draw_status_indicator;

fn draw_menu_items(ui: &mut egui::Ui, items: &[MenuItem]) {
    for item in items {
        match item {
            MenuItem::Action { label, callback } => {
                if ui.button(label).clicked() {
                    (callback)();
                    ui.close();
                }
            }
            MenuItem::Submenu { label, items } => {
                ui.menu_button(label, |ui| {
                    draw_menu_items(ui, items);
                });
            }
            MenuItem::Separator => {
                ui.separator();
            }
        }
    }
}

pub fn draw_top_bar(ctx: &egui::Context, ui: &mut egui::Ui, registry: &MenuRegistry, compile_status: &CompileStatus) {
    egui::Panel::top("main_top_bar")
        .frame(egui::Frame::new().fill(egui::Color32::from_rgb(30, 30, 30)))
        .show(ui, |ui| {
            ui.set_min_height(18.0);
            ui.horizontal(|ui| {
                for menu in &registry.menus {
                    ui.menu_button(&menu.label, |ui| {
                        draw_menu_items(ui, &menu.items);
                    });
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    draw_status_indicator(ctx, ui, compile_status);
                });
            });
        });
}
