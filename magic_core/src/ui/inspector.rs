use crate::api::assets::global_asset_registry;
use crate::api::window::{EditorWindow, WindowId};
use eframe::egui::{self, Color32, RichText, Stroke};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};


const BG_DEEP: Color32 = Color32::from_rgb(12, 13, 18);
const BG_PANEL: Color32 = Color32::from_rgb(22, 24, 31);
const BG_BAR: Color32 = Color32::from_rgb(28, 31, 40);
const ITEM_BG: Color32 = Color32::from_rgb(30, 33, 43);
const BORDER_COLOR: Color32 = Color32::from_rgb(45, 50, 65);
const ACCENT_PURPLE: Color32 = Color32::from_rgb(139, 92, 246);
const ACCENT_CYAN: Color32 = Color32::from_rgb(56, 189, 248);
const ACCENT_GOLD: Color32 = Color32::from_rgb(245, 185, 66);
const TEXT_MAIN: Color32 = Color32::from_rgb(230, 233, 240);
const TEXT_MUTED: Color32 = Color32::from_rgb(140, 147, 168);

#[derive(Clone)]
pub enum InspectorTarget {
    File(PathBuf),
    Custom(String),
}

type InspectorDrawFn = Box<dyn Fn(&mut egui::Ui) + Send + Sync>;

static INSPECTOR_TARGET: OnceLock<Arc<Mutex<Option<InspectorTarget>>>> = OnceLock::new();
static INSPECTOR_REGISTRY: OnceLock<Arc<Mutex<HashMap<String, InspectorDrawFn>>>> = OnceLock::new();

fn target_store() -> &'static Arc<Mutex<Option<InspectorTarget>>> {
    INSPECTOR_TARGET.get_or_init(|| Arc::new(Mutex::new(None)))
}

fn inspector_registry() -> &'static Arc<Mutex<HashMap<String, InspectorDrawFn>>> {
    INSPECTOR_REGISTRY.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

pub fn set_inspector_target(target: InspectorTarget) {
    if let Ok(mut t) = target_store().lock() {
        *t = Some(target);
    }
}

pub fn set_inspector_file(path: PathBuf) {
    set_inspector_target(InspectorTarget::File(path));
}

pub fn get_inspector_target() -> Option<InspectorTarget> {
    target_store().lock().ok().and_then(|t| t.clone())
}

pub fn register_inspector(id: impl Into<String>, draw: InspectorDrawFn) {
    if let Ok(mut reg) = inspector_registry().lock() {
        reg.insert(id.into(), draw);
    }
}

pub fn unregister_inspector(id: &str) {
    if let Ok(mut reg) = inspector_registry().lock() {
        reg.remove(id);
    }
}

pub fn set_inspector_target_file(path: PathBuf) {
    set_inspector_target(InspectorTarget::File(path));
}

pub struct InspectorWindow {
    locked: bool,
    frozen: Option<InspectorTarget>,
}

impl InspectorWindow {
    pub fn new() -> Self {
        Self {
            locked: false,
            frozen: None,
        }
    }

    fn format_size(bytes: u64) -> String {
        const KB: f64 = 1024.0;
        const MB: f64 = KB * 1024.0;
        const GB: f64 = MB * 1024.0;

        let b = bytes as f64;
        if b >= GB {
            format!("{:.2} GB ({} B)", b / GB, bytes)
        } else if b >= MB {
            format!("{:.2} MB ({} B)", b / MB, bytes)
        } else if b >= KB {
            format!("{:.1} KB ({} B)", b / KB, bytes)
        } else {
            format!("{} B", bytes)
        }
    }

    fn fallback_icon_and_meta(path: &Path) -> (String, String, Color32) {
        if path.is_dir() {
            return (
                egui_phosphor::regular::FOLDER.to_string(),
                "Arcane Directory".to_string(),
                ACCENT_GOLD,
            );
        }

        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => (
                egui_phosphor::regular::FILE_RS.to_string(),
                "Rust Spell Script".to_string(),
                Color32::from_rgb(251, 146, 60),
            ),
            "toml" | "ini" | "cfg" | "yaml" | "yml" => (
                egui_phosphor::regular::GEAR.to_string(),
                "Config Artifact".to_string(),
                ACCENT_CYAN,
            ),
            "json" | "ron" => (
                egui_phosphor::regular::BRACKETS_CURLY.to_string(),
                "Rune Data Scroll".to_string(),
                ACCENT_GOLD,
            ),
            "ogg" | "wav" | "mp3" | "flac" => (
                egui_phosphor::regular::MUSIC_NOTES.to_string(),
                "Bardic Audio".to_string(),
                Color32::from_rgb(244, 114, 182),
            ),
            "png" | "jpg" | "jpeg" | "webp" | "svg" => (
                egui_phosphor::regular::IMAGE.to_string(),
                "Visual Texture".to_string(),
                Color32::from_rgb(52, 211, 153),
            ),
            "wgsl" | "glsl" | "vert" | "frag" => (
                egui_phosphor::regular::SPARKLE.to_string(),
                "Arcane Shader".to_string(),
                ACCENT_PURPLE,
            ),
            _ => (
                egui_phosphor::regular::FILE.to_string(),
                if ext.is_empty() {
                    "Unknown Artifact".to_string()
                } else {
                    format!(".{} File", ext.to_uppercase())
                },
                TEXT_MUTED,
            ),
        }
    }

    fn draw_top_bar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(BG_BAR)
            .stroke(Stroke::new(1.0, BORDER_COLOR))
            .inner_margin(egui::Margin::symmetric(8, 5))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (status_icon, status_text, status_color) = if self.locked {
                        (
                            egui_phosphor::regular::LOCK_KEY,
                            "Pinned Target",
                            ACCENT_GOLD,
                        )
                    } else {
                        (egui_phosphor::regular::SPARKLE, "", ACCENT_PURPLE)
                    };

                    ui.label(RichText::new(status_icon).color(status_color).size(14.0));
                    ui.label(
                        RichText::new(status_text)
                            .color(if self.locked { ACCENT_GOLD } else { TEXT_MAIN })
                            .strong()
                            .size(12.0),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let pin_icon = if self.locked {
                            egui_phosphor::regular::PUSH_PIN
                        } else {
                            egui_phosphor::regular::PUSH_PIN_SLASH
                        };

                        let btn_fill = if self.locked {
                            Color32::from_rgba_unmultiplied(245, 185, 66, 38)
                        } else {
                            ITEM_BG
                        };

                        let btn_stroke = if self.locked {
                            Stroke::new(1.0, ACCENT_GOLD)
                        } else {
                            Stroke::new(1.0, BORDER_COLOR)
                        };

                        let btn_text_color = if self.locked { ACCENT_GOLD } else { TEXT_MUTED };

                        let btn = egui::Button::new(
                            RichText::new(pin_icon).color(btn_text_color).size(14.0),
                        )
                        .fill(btn_fill)
                        .stroke(btn_stroke);

                        let tooltip = if self.locked {
                            "Unpin Inspector (Follow Selection)"
                        } else {
                            "Pin Inspector to Current Target"
                        };

                        if ui.add(btn).on_hover_text(tooltip).clicked() {
                            self.locked = !self.locked;
                        }
                    });
                });
            });
    }

    fn draw_default_inspector(ui: &mut egui::Ui, path: &Path) {
        egui::Frame::group(ui.style())
            .fill(BG_PANEL)
            .stroke(Stroke::new(1.0, BORDER_COLOR))
            .inner_margin(10.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(egui_phosphor::regular::SLIDERS_HORIZONTAL)
                            .color(ACCENT_CYAN)
                            .size(14.0),
                    );
                    ui.label(
                        RichText::new("Properties")
                            .color(TEXT_MAIN)
                            .strong()
                            .size(12.5),
                    );
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                egui::Grid::new("default_inspector_props")
                    .num_columns(2)
                    .spacing([14.0, 8.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label(RichText::new("Name").color(TEXT_MUTED).size(11.5));
                        ui.add(
                            egui::Label::new(
                                RichText::new(
                                    path.file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("Unknown"),
                                )
                                .color(TEXT_MAIN)
                                .strong()
                                .size(12.0),
                            )
                            .truncate(),
                        );
                        ui.end_row();

                        ui.label(RichText::new("Type").color(TEXT_MUTED).size(11.5));
                        let type_str = if path.is_dir() {
                            "Folder".to_string()
                        } else {
                            path.extension()
                                .and_then(|s| s.to_str())
                                .map(|e| format!(".{} Asset", e.to_uppercase()))
                                .unwrap_or_else(|| "File".to_string())
                        };
                        ui.label(RichText::new(type_str).color(ACCENT_CYAN).size(11.5));
                        ui.end_row();

                        if path.is_file() {
                            if let Ok(meta) = std::fs::metadata(path) {
                                ui.label(RichText::new("Size").color(TEXT_MUTED).size(11.5));
                                ui.label(
                                    RichText::new(Self::format_size(meta.len()))
                                        .color(TEXT_MAIN)
                                        .size(11.5),
                                );
                                ui.end_row();
                            }
                        } else if path.is_dir() {
                            if let Ok(entries) = std::fs::read_dir(path) {
                                let count = entries.count();
                                ui.label(RichText::new("Contents").color(TEXT_MUTED).size(11.5));
                                ui.label(
                                    RichText::new(format!("{} items", count))
                                        .color(TEXT_MAIN)
                                        .size(11.5),
                                );
                                ui.end_row();
                            }
                        }

                        ui.label(RichText::new("Path").color(TEXT_MUTED).size(11.5));
                        ui.horizontal(|ui| {
                            let path_str = path.display().to_string();
                            if ui
                                .small_button(egui_phosphor::regular::COPY)
                                .on_hover_text("Copy Full Path")
                                .clicked()
                            {
                                ui.ctx().copy_text(path_str.clone());
                            }
                            ui.add(
                                egui::Label::new(
                                    RichText::new(path_str)
                                        .monospace()
                                        .color(TEXT_MUTED)
                                        .size(11.0),
                                )
                                .truncate(),
                            );
                        });
                        ui.end_row();
                    });
            });

        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();

            if matches!(
                ext.as_str(),
                "rs" | "toml" | "json" | "ron" | "txt" | "md" | "wgsl" | "yaml" | "yml" | "cfg"
            ) {
                if let Ok(content) = std::fs::read_to_string(path) {
                    ui.add_space(8.0);

                    egui::Frame::group(ui.style())
                        .fill(BG_PANEL)
                        .stroke(Stroke::new(1.0, BORDER_COLOR))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            let total_lines = content.lines().count();
                            let preview_lines: Vec<&str> = content.lines().take(120).collect();

                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(egui_phosphor::regular::CODE)
                                        .color(ACCENT_PURPLE)
                                        .size(14.0),
                                );
                                ui.label(
                                    RichText::new("Scroll Preview")
                                        .color(TEXT_MAIN)
                                        .strong()
                                        .size(12.5),
                                );

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.label(
                                            RichText::new(format!("{} lines", total_lines))
                                                .color(TEXT_MUTED)
                                                .size(11.0),
                                        );
                                    },
                                );
                            });

                            ui.add_space(6.0);

                            egui::Frame::group(ui.style())
                                .fill(BG_DEEP)
                                .stroke(Stroke::new(1.0, BORDER_COLOR))
                                .inner_margin(8.0)
                                .show(ui, |ui| {
                                    egui::ScrollArea::both()
                                        .max_height(320.0)
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            ui.horizontal_top(|ui| {
                                                let line_nums: String = (1..=preview_lines.len())
                                                    .map(|n| format!("{:>3}", n))
                                                    .collect::<Vec<_>>()
                                                    .join("\n");

                                                ui.label(
                                                    RichText::new(line_nums)
                                                        .monospace()
                                                        .color(Color32::from_rgb(75, 85, 105))
                                                        .size(11.5),
                                                );

                                                ui.separator();

                                               
                                                let code_text = preview_lines.join("\n");
                                                ui.label(
                                                    RichText::new(code_text)
                                                        .monospace()
                                                        .color(TEXT_MAIN)
                                                        .size(11.5),
                                                );
                                            });
                                        });
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
        self.draw_top_bar(ui);
        ui.add_space(6.0);

        if !self.locked {
            self.frozen = get_inspector_target();
        }

        let Some(target) = self.frozen.clone() else {
            ui.add_space(36.0);
            ui.vertical_centered(|ui| {
                egui::Frame::group(ui.style())
                    .fill(BG_PANEL)
                    .stroke(Stroke::new(1.0, BORDER_COLOR))
                    .inner_margin(20.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(egui_phosphor::regular::MAGNIFYING_GLASS)
                                    .size(30.0)
                                    .color(TEXT_MUTED),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new("No Artifact Selected")
                                    .strong()
                                    .color(TEXT_MAIN)
                                    .size(13.5),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Select a file or folder in Grimoire to inspect.")
                                    .color(TEXT_MUTED)
                                    .size(11.5),
                            );
                        });
                    });
            });
            return;
        };

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| match target {
                InspectorTarget::File(path) => {
                    if !path.exists() {
                        egui::Frame::group(ui.style())
                            .fill(BG_PANEL)
                            .stroke(Stroke::new(1.0, Color32::from_rgb(239, 68, 68)))
                            .inner_margin(12.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(egui_phosphor::regular::WARNING_CIRCLE)
                                            .color(Color32::from_rgb(239, 68, 68))
                                            .size(18.0),
                                    );
                                    ui.label(
                                        RichText::new("Selected item no longer exists.")
                                            .color(TEXT_MAIN),
                                    );
                                });
                            });
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

                    let (icon, subtitle, pkg, color) = custom_header.unwrap_or_else(|| {
                        let (ic, sub, col) = Self::fallback_icon_and_meta(&path);
                        (ic, sub, None, col)
                    });

                    egui::Frame::group(ui.style())
                        .fill(ITEM_BG)
                        .stroke(Stroke::new(1.0, BORDER_COLOR))
                        .inner_margin(10.0)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                egui::Frame::group(ui.style())
                                    .fill(BG_DEEP)
                                    .stroke(Stroke::new(1.0, color))
                                    .inner_margin(8.0)
                                    .show(ui, |ui| {
                                        ui.label(RichText::new(icon).size(22.0).color(color));
                                    });

                                ui.add_space(6.0);

                                ui.vertical(|ui| {
                                    let name = path
                                        .file_name()
                                        .and_then(|s| s.to_str())
                                        .unwrap_or("Asset");

                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(name)
                                                .strong()
                                                .color(TEXT_MAIN)
                                                .size(14.5),
                                        )
                                        .truncate(),
                                    );

                                    ui.add_space(2.0);

                                    ui.horizontal_wrapped(|ui| {
                                        ui.label(
                                            RichText::new(subtitle)
                                                .size(11.5)
                                                .strong()
                                                .color(color),
                                        );
                                        if let Some(p) = pkg {
                                            ui.label(
                                                RichText::new(format!("• {}", p))
                                                    .size(11.0)
                                                    .color(TEXT_MUTED),
                                            );
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
                                egui::Frame::group(ui.style())
                                    .fill(BG_PANEL)
                                    .stroke(Stroke::new(1.0, BORDER_COLOR))
                                    .inner_margin(10.0)
                                    .show(ui, |ui| {
                                        (inspect_fn)(ui, &path);
                                    });
                            }
                        }
                    }

                    if !handled_by_package {
                        Self::draw_default_inspector(ui, &path);
                    }
                }
                InspectorTarget::Custom(id) => {
                    if let Ok(reg) = inspector_registry().lock() {
                        if let Some(draw_fn) = reg.get(&id) {
                            egui::Frame::group(ui.style())
                                .fill(BG_PANEL)
                                .stroke(Stroke::new(1.0, BORDER_COLOR))
                                .inner_margin(10.0)
                                .show(ui, |ui| {
                                    (draw_fn)(ui);
                                });
                        } else {
                            ui.weak(format!("No inspector registered for '{}'", id));
                        }
                    }
                }
            });
    }
}
