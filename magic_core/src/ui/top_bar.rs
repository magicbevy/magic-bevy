use magic_api_editor::api::{MenuItem, MenuRegistry};
use magic_api_editor::status::CompileStatus;
use crate::ui::indicator::draw_status_indicator;
use eframe::egui::{self, RichText};

#[derive(Debug, Clone)]
pub enum LayoutAction {
    None,
    Save { name: String, is_global: bool },
    Load { name: String, is_global: bool },
    Delete { name: String, is_global: bool },
}

fn draw_menu_items(ui: &mut egui::Ui, items: &[MenuItem]) {
    ui.set_min_width(150.0);
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

    let top_frame = egui::Frame::new()
        .fill(ui.visuals().panel_fill)
        .inner_margin(egui::Margin::symmetric(10, 5))
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke);

    egui::Panel::top("main_top_bar")
        .frame(top_frame)
        .show(ui, |ui| {
            ui.set_min_height(24.0);
            ui.horizontal(|ui| {

                for menu in &registry.menus {
                    ui.menu_button(&menu.label, |ui| {
                        draw_menu_items(ui, &menu.items);
                    });
                }


                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    draw_status_indicator(ctx, ui, compile_status);

                    ui.separator();

                    if ui
                        .button(format!("{} Save Layout", egui_phosphor::regular::FLOPPY_DISK))
                        .on_hover_text("Save current workspace layout")
                        .clicked()
                    {
                        *show_save_dialog = true;
                        *save_name = current_layout.clone().unwrap_or_default();
                    }


                    let selected_label = match current_layout.as_deref() {
                        Some(name) if !name.is_empty() => format!("{} {}", egui_phosphor::regular::RULER, name),
                        _ => format!("{} Layouts", egui_phosphor::regular::RULER),
                    };

                    egui::ComboBox::from_id_salt("workspace_layout_combo")
                        .selected_text(selected_label)
                        .width(150.0)
                        .show_ui(ui, |ui| {
                            if project_layouts.is_empty() && global_layouts.is_empty() {
                                ui.weak("No saved layouts");
                            }

                            if !project_layouts.is_empty() {
                                ui.weak(format!("{} Project Layouts", egui_phosphor::regular::FOLDER));
                                for name in project_layouts {
                                    let is_selected = current_layout.as_ref() == Some(name);
                                    ui.horizontal(|ui| {
                                        if ui
                                            .selectable_label(is_selected, format!("  {} {}", egui_phosphor::regular::FOLDER, name))
                                            .clicked()
                                        {
                                            *current_layout = Some(name.clone());
                                            action = LayoutAction::Load {
                                                name: name.clone(),
                                                is_global: false,
                                            };
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .small_button(egui_phosphor::regular::TRASH)
                                                    .on_hover_text("Delete layout")
                                                    .clicked()
                                                {
                                                    *delete_confirm = Some((name.clone(), false));
                                                }
                                            },
                                        );
                                    });
                                }
                                if !global_layouts.is_empty() {
                                    ui.separator();
                                }
                            }

                            if !global_layouts.is_empty() {
                                ui.weak(format!("{} Global Layouts", egui_phosphor::regular::GLOBE));
                                for name in global_layouts {
                                    let is_selected = current_layout.as_ref() == Some(name);
                                    ui.horizontal(|ui| {
                                        if ui
                                            .selectable_label(
                                                is_selected,
                                                format!("  {} {}", egui_phosphor::regular::GLOBE, name),
                                            )
                                            .clicked()
                                        {
                                            *current_layout = Some(name.clone());
                                            action = LayoutAction::Load {
                                                name: name.clone(),
                                                is_global: true,
                                            };
                                        }

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui
                                                    .small_button(egui_phosphor::regular::TRASH)
                                                    .on_hover_text("Delete layout")
                                                    .clicked()
                                                {
                                                    *delete_confirm = Some((name.clone(), true));
                                                }
                                            },
                                        );
                                    });
                                }
                            }
                        });
                });
            });
        });


    if *show_save_dialog {
        let scope_id = egui::Id::new("save_layout_is_global");
        let mut is_global = ctx.data_mut(|d| *d.get_temp_mut_or(scope_id, true));

        egui::Window::new(format!("{} Save Workspace Layout", egui_phosphor::regular::FLOPPY_DISK))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(280.0);
                ui.label("Enter a name for this layout:");
                let resp = ui.add(
                    egui::TextEdit::singleline(save_name)
                        .hint_text("e.g., Debugging, Level Design...")
                        .desired_width(f32::INFINITY),
                );
                if resp.gained_focus() {
                    resp.request_focus();
                }

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.label("Scope:");
                    ui.radio_value(&mut is_global, true, format!("{} Global", egui_phosphor::regular::GLOBE));
                    ui.radio_value(&mut is_global, false, format!("{} Project", egui_phosphor::regular::FOLDER));
                });
                ctx.data_mut(|d| d.insert_temp(scope_id, is_global));

                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    let can_save = !save_name.trim().is_empty();
                    let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter)) && can_save;

                    if (ui
                        .add_enabled(can_save, egui::Button::new(format!("{} Save", egui_phosphor::regular::CHECK)))
                        .clicked()
                        || enter_pressed)
                        && can_save
                    {
                        let trimmed = save_name.trim().to_string();
                        *current_layout = Some(trimmed.clone());
                        action = LayoutAction::Save {
                            name: trimmed,
                            is_global,
                        };
                        *show_save_dialog = false;
                    }

                    if ui.button("Cancel").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        *show_save_dialog = false;
                    }
                });
            });
    }


    if let Some((name, is_global)) = delete_confirm.as_ref() {
        let name_clone = name.clone();
        let is_global_clone = *is_global;

        egui::Window::new(format!("{} Delete Layout", egui_phosphor::regular::TRASH))
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(260.0);
                ui.label(format!(
                    "Are you sure you want to delete layout '{}'?",
                    name_clone
                ));
                ui.add_space(10.0);
                ui.separator();
                ui.horizontal(|ui| {
                    let del_btn = egui::Button::new(
                        RichText::new(format!("{} Delete", egui_phosphor::regular::TRASH)).color(ui.visuals().error_fg_color),
                    );
                    if ui.add(del_btn).clicked() {
                        action = LayoutAction::Delete {
                            name: name_clone,
                            is_global: is_global_clone,
                        };
                        *delete_confirm = None;
                    }
                    if ui.button("Cancel").clicked() {
                        *delete_confirm = None;
                    }
                });
            });
    }

    action
}