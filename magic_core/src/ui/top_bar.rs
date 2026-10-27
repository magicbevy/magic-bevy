use eframe::egui;
use crate::api::{MenuItem, MenuRegistry};
use crate::status::CompileStatus;
use crate::ui::indicator::draw_status_indicator;

#[derive(Debug, Clone)]
pub enum LayoutAction {
    None,
    Save { name: String, is_global: bool },
    Load { name: String, is_global: bool },
    Delete { name: String, is_global: bool },
}

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
                    draw_menu_items(ui, &items);
                });
            }
            MenuItem::Separator => {
                ui.separator();
            }
        }
    }
}

pub fn draw_top_bar(
    ctx: &egui::Context,
    ui: &mut egui::Ui,
    registry: &MenuRegistry,
    compile_status: &CompileStatus,
    project_layouts: &[String],
    global_layouts: &[String],
    current_layout: &mut Option<String>,
    save_name: &mut String,
    show_save_dialog: &mut bool,
    delete_confirm: &mut Option<(String, bool)>,
) -> LayoutAction {
    let mut action = LayoutAction::None;

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

                    ui.add_space(10.0);

                    if ui.button("💾 Save").clicked() {
                        *show_save_dialog = true;
                        *save_name = String::new();
                    }

                    if *show_save_dialog {
                        egui::Window::new("Save Layout")
                            .collapsible(false)
                            .resizable(false)
                            .show(ctx, |ui| {
                                ui.label("Layout name:");
                                ui.text_edit_singleline(save_name);
                                ui.horizontal(|ui| {
                                    if ui.button("OK").clicked() && !save_name.is_empty() {
                                        action = LayoutAction::Save {
                                            name: save_name.clone(),
                                            is_global: true,
                                        };
                                        *show_save_dialog = false;
                                    }
                                    if ui.button("Cancel").clicked() {
                                        *show_save_dialog = false;
                                    }
                                });
                            });
                    }

                    ui.add_space(5.0);

                    let mut all_items: Vec<(String, bool)> = Vec::new();
                    for name in project_layouts {
                        all_items.push((format!("📁 {}", name), false));
                    }
                    for name in global_layouts {
                        all_items.push((format!("🌐 {}", name), true));
                    }

                    egui::ComboBox::from_label("")
                        .selected_text("Layout")
                        .show_ui(ui, |ui| {
                            for (display_name, is_global) in &all_items {
                                let real_name = display_name
                                    .strip_prefix("📁 ")
                                    .or_else(|| display_name.strip_prefix("🌐 "))
                                    .unwrap_or(display_name)
                                    .to_string();
                                let is_selected = current_layout.as_ref() == Some(&real_name);
                                ui.horizontal(|ui| {
                                    if ui.selectable_label(is_selected, display_name.as_str()).clicked() {
                                        *current_layout = Some(real_name.clone());
                                        action = LayoutAction::Load {
                                            name: real_name,
                                            is_global: *is_global,
                                        };
                                    }

                                    if *is_global && ui.button("−").clicked() {
                                        *delete_confirm = Some((name_from_display(display_name), *is_global));
                                    }
                                });
                            }
                        });
                });
            });
        });

    if let Some((name, is_global)) = delete_confirm.as_ref() {
        let name_clone = name.clone();
        let is_global_clone = *is_global;
        egui::Window::new("Delete Layout")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Delete layout '{}'?", name_clone));
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        action = LayoutAction::Delete {
                            name: name_clone,
                            is_global: is_global_clone,
                        };
                        *delete_confirm = None;
                    }
                    if ui.button("No").clicked() {
                        *delete_confirm = None;
                        action = LayoutAction::None;
                    }
                });
            });
    }

    action
}

fn name_from_display(display_name: &str) -> String {
    display_name
        .strip_prefix("📁 ")
        .or_else(|| display_name.strip_prefix("🌐 "))
        .unwrap_or(display_name)
        .to_string()
}
