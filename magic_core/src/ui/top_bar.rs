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
                    ui.close_menu();
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

pub fn draw_top_bar(ctx: &egui::Context, registry: &MenuRegistry, compile_status: &CompileStatus) {
    egui::TopBottomPanel::top("main_top_bar").show(ctx, |ui| {
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
