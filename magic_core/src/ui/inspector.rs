use crate::api::assets::global_asset_registry;
use crate::api::window::{EditorWindow, WindowId};
use eframe::egui::{self, RichText};
use std::path::Path;

pub struct InspectorWindow;

impl InspectorWindow {
    pub fn new() -> Self {
        Self
    }

    fn draw_default_inspector(ui: &mut egui::Ui, path: &Path) {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.label(RichText::new("Properties").strong());
                ui.separator();

                egui::Grid::new("default_inspector_props")
                    .num_columns(2)
                    .spacing([16.0, 6.0])
                    .show(ui, |ui| {
                        ui.weak("Name:");
                        ui.label(
                            path.file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Unknown"),
                        );
                        ui.end_row();

                        ui.weak("Type:");
                        ui.label(if path.is_dir() {
                            "Folder".to_string()
                        } else {
                            path.extension()
                                .and_then(|s| s.to_str())
                                .map(|e| format!(".{} file", e))
                                .unwrap_or_else(|| "File".to_string())
                        });
                        ui.end_row();

                        if path.is_file() {
                            if let Ok(meta) = std::fs::metadata(path) {
                                ui.weak("Size:");
                                ui.label(format!("{} bytes", meta.len()));
                                ui.end_row();
                            }
                        }

                        ui.weak("Path:");
                        ui.label(RichText::new(path.display().to_string()).monospace().size(11.0));
                        ui.end_row();
                    });
            });

        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            if matches!(ext.as_str(), "rs" | "toml" | "json" | "ron" | "txt" | "md" | "wgsl") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    ui.add_space(8.0);
                    ui.label(RichText::new("File Preview").strong());
                    egui::Frame::group(ui.style())
                        .fill(ui.visuals().extreme_bg_color)
                        .inner_margin(8.0)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical()
                                .max_height(300.0)
                                .show(ui, |ui| {
                                    let preview: String =
                                        content.lines().take(100).collect::<Vec<_>>().join("\n");
                                    ui.label(RichText::new(preview).monospace().size(11.5));
                                });
                        });
                }
            }
        }
    }
}

impl EditorWindow for InspectorWindow {
    fn id(&self) -> WindowId {
        "inspector".to_string()
    }

    fn title(&self) -> &str {
        "Inspector"
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        let selected_path = global_asset_registry()
            .lock()
            .ok()
            .and_then(|reg| reg.selected_path().cloned());

        let Some(path) = selected_path else {
            ui.add_space(40.0);
            ui.vertical_centered(|ui| {
                ui.weak("Select a file or folder in Grimoire to inspect.");
            });
            return;
        };

        if !path.exists() {
            ui.weak("Selected item no longer exists.");
            return;
        }

        let mut custom_header = None;
        if let Ok(reg) = global_asset_registry().lock() {
            if let Some(desc) = reg.get_by_path(&path) {
                custom_header = Some((
                    desc.icon.clone(),
                    desc.display_name.clone(),
                    desc.package_name.clone(),
                    desc.color,
                ));
            }
        }

        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (icon, subtitle, pkg, color) = custom_header.unwrap_or_else(|| {
                        let ic = if path.is_dir() { egui_phosphor::regular::FOLDER } else { egui_phosphor::regular::FILE };
                        (
                            ic.to_string(),
                            if path.is_dir() {
                                "Directory".to_string()
                            } else {
                                "File Asset".to_string()
                            },
                            None,
                            ui.visuals().hyperlink_color,
                        )
                    });

                    ui.label(RichText::new(icon).size(26.0));
                    ui.vertical(|ui| {
                        let name = path
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Asset");
                        ui.label(RichText::new(name).strong().size(15.0));
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(subtitle).size(11.5).color(color));
                            if let Some(p) = pkg {
                                ui.weak(format!("• Package: {}", p));
                            }
                        });
                    });
                });
            });

        ui.add_space(8.0);

        let mut handled_by_package = false;
        if let Ok(mut reg) = global_asset_registry().lock() {
            if let Some(desc) = reg.get_mut_by_path(&path) {
                if let Some(inspect_fn) = desc.on_inspect.as_mut() {
                    handled_by_package = true;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        (inspect_fn)(ui, &path);
                    });
                }
            }
        }

        if !handled_by_package {
            Self::draw_default_inspector(ui, &path);
        }
    }
}