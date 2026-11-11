use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

use magic_log::{global_store, LogLevel};
use chrono::Timelike;
use eframe::egui::{self, Color32, RichText, Sense, Stroke};

fn copy_to_clipboard(ctx: &egui::Context, text: &str) {
    ctx.copy_text(text.to_string());

    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }
    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

pub struct OracleLogsWindow {
    items: Vec<LogItem>,
    expanded: HashSet<usize>,
    selected: HashSet<usize>,
    last_store_len: usize,
    search_query: String,
    auto_scroll: bool,
    filter_info: bool,
    filter_warn: bool,
    filter_error: bool,
    filter_success: bool,
    filter_debug: bool,
    show_timestamps: bool,

    count_info: usize,
    count_warn: usize,
    count_error: usize,
    count_success: usize,
    count_debug: usize,
}

struct LogItem {
    id: usize,
    time: String,
    level: LogLevel,
    title: String,
    file_line: Option<String>,
    body: String,
    line_count: usize,
}

impl OracleLogsWindow {
    fn level_color(level: &LogLevel) -> egui::Color32 {
        match level {
            LogLevel::Info => egui::Color32::from_rgb(100, 200, 255),
            LogLevel::Warn => egui::Color32::from_rgb(255, 200, 50),
            LogLevel::Error => egui::Color32::from_rgb(255, 80, 80),
            LogLevel::Success => egui::Color32::from_rgb(100, 255, 100),
            LogLevel::Debug => egui::Color32::from_rgb(200, 200, 200),
        }
    }

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            expanded: HashSet::new(),
            selected: HashSet::new(),
            last_store_len: 0,
            search_query: String::new(),
            auto_scroll: true,
            filter_info: true,
            filter_warn: true,
            filter_error: true,
            filter_success: true,
            filter_debug: true,
            show_timestamps: true,
            count_info: 0,
            count_warn: 0,
            count_error: 0,
            count_success: 0,
            count_debug: 0,
        }
    }

    fn refresh_from_store(&mut self) {
        let store = global_store().lock().ok();
        let entries = match store {
            Some(s) => s.entries().to_vec(),
            None => Vec::new(),
        };

        if entries.len() == self.last_store_len && !self.items.is_empty() {
            return;
        }

        if entries.len() < self.last_store_len {

            self.expanded.clear();
            self.selected.clear();
        }

        self.last_store_len = entries.len();
        self.items.clear();

        self.count_info = 0;
        self.count_warn = 0;
        self.count_error = 0;
        self.count_success = 0;
        self.count_debug = 0;

        for (id, entry) in entries.into_iter().enumerate() {
            match entry.level {
                LogLevel::Info => self.count_info += 1,
                LogLevel::Warn => self.count_warn += 1,
                LogLevel::Error => self.count_error += 1,
                LogLevel::Success => self.count_success += 1,
                LogLevel::Debug => self.count_debug += 1,
            }

            let time = format!(
                "{:02}:{:02}:{:02}",
                entry.timestamp.hour(),
                entry.timestamp.minute(),
                entry.timestamp.second()
            );

            let first_line = entry
                .message
                .lines()
                .next()
                .unwrap_or(&entry.message)
                .to_string();
            let file_line = extract_file_line(&entry.message);
            let line_count = entry.message.lines().count();

            self.items.push(LogItem {
                id,
                time,
                level: entry.level,
                title: first_line,
                file_line,
                body: entry.message,
                line_count,
            });
        }
    }

    fn should_show(&self, item: &LogItem) -> bool {
        let level_ok = match item.level {
            LogLevel::Info => self.filter_info,
            LogLevel::Warn => self.filter_warn,
            LogLevel::Error => self.filter_error,
            LogLevel::Success => self.filter_success,
            LogLevel::Debug => self.filter_debug,
        };

        if !level_ok {
            return false;
        }

        if !self.search_query.trim().is_empty() {
            let q = self.search_query.to_lowercase();
            let in_body = item.body.to_lowercase().contains(&q);
            let in_file = item
                .file_line
                .as_ref()
                .map(|f| f.to_lowercase().contains(&q))
                .unwrap_or(false);
            if !in_body && !in_file {
                return false;
            }
        }

        true
    }

    fn level_icon(&self, level: &LogLevel) -> &'static str {
        match level {
            LogLevel::Error => egui_phosphor::regular::X_CIRCLE,
            LogLevel::Warn => egui_phosphor::regular::WARNING,
            LogLevel::Info => "ℹ",
            LogLevel::Success => egui_phosphor::regular::CHECK_CIRCLE,
            LogLevel::Debug => egui_phosphor::regular::GEAR,
        }
    }

    fn draw_filter_pill(
        ui: &mut egui::Ui,
        active: &mut bool,
        icon: &str,
        label: &str,
        count: usize,
        color: Color32,
    ) {
        let text = format!("{} {} ({})", icon, label, count);
        let text_color = if *active {
            color
        } else {
            ui.visuals().weak_text_color()
        };

        let fill = if *active {
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 28)
        } else {
            ui.visuals().widgets.inactive.bg_fill
        };

        let stroke = if *active {
            Stroke::new(
                1.0,
                Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 140),
            )
        } else {
            ui.visuals().widgets.noninteractive.bg_stroke
        };

        let btn = egui::Button::new(RichText::new(text).size(11.5).strong().color(text_color))
            .fill(fill)
            .stroke(stroke);

        if ui.add(btn).clicked() {
            *active = !*active;
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .inner_margin(egui::Margin::symmetric(8, 6))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    if ui
                        .button(format!("{} Clear", egui_phosphor::regular::TRASH))
                        .on_hover_text("Clear all console logs")
                        .clicked()
                    {
                        if let Ok(mut store) = global_store().lock() {
                            store.clear();
                        }
                        self.items.clear();
                        self.expanded.clear();
                        self.selected.clear();
                        self.last_store_len = 0;
                        self.count_info = 0;
                        self.count_warn = 0;
                        self.count_error = 0;
                        self.count_success = 0;
                        self.count_debug = 0;
                    }

                    ui.separator();


                    ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text(format!("{} Search logs or files...", egui_phosphor::regular::MAGNIFYING_GLASS))
                            .desired_width(180.0),
                    );
                    if !self.search_query.is_empty() && ui.small_button(egui_phosphor::regular::X).clicked() {
                        self.search_query.clear();
                    }

                    ui.separator();


                    Self::draw_filter_pill(
                        ui,
                        &mut self.filter_error,
                        egui_phosphor::regular::X_CIRCLE,
                        "ERR",
                        self.count_error,
                        Self::level_color(&LogLevel::Error),
                    );
                    Self::draw_filter_pill(
                        ui,
                        &mut self.filter_warn,
                        egui_phosphor::regular::WARNING,
                        "WARN",
                        self.count_warn,
                        Self::level_color(&LogLevel::Warn),
                    );
                    Self::draw_filter_pill(
                        ui,
                        &mut self.filter_info,
                        "ℹ",
                        "INFO",
                        self.count_info,
                        Self::level_color(&LogLevel::Info),
                    );
                    Self::draw_filter_pill(
                        ui,
                        &mut self.filter_success,
                        egui_phosphor::regular::CHECK_CIRCLE,
                        "OK",
                        self.count_success,
                        Self::level_color(&LogLevel::Success),
                    );
                    Self::draw_filter_pill(
                        ui,
                        &mut self.filter_debug,
                        egui_phosphor::regular::GEAR,
                        "DBG",
                        self.count_debug,
                        Self::level_color(&LogLevel::Debug),
                    );

                    ui.separator();

                    ui.checkbox(&mut self.show_timestamps, format!("{} Time", egui_phosphor::regular::CLOCK));
                    ui.checkbox(&mut self.auto_scroll, "⬇ Auto-scroll");
                });
            });
    }

    fn draw_logs(&mut self, ui: &mut egui::Ui) {
        self.refresh_from_store();

        let visible_indices: Vec<usize> = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| self.should_show(item))
            .map(|(idx, _)| idx)
            .collect();

        let available_height = (ui.available_height() - 32.0).max(60.0);

        egui::Frame::group(ui.style())
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::same(4))
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(available_height)
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        if visible_indices.is_empty() {
                            ui.add_space(32.0);
                            ui.vertical_centered(|ui| {
                                ui.weak(format!("{} No log entries match the active filters.", egui_phosphor::regular::FILE_TEXT));
                            });
                            return;
                        }

                        for (row_num, &i) in visible_indices.iter().enumerate() {
                            let item = &self.items[i];
                            let id = item.id;
                            let is_expanded = self.expanded.contains(&id);
                            let is_selected = self.selected.contains(&id);
                            let marker = if is_expanded { "▾" } else { "▸" };
                            let level_color = Self::level_color(&item.level);
                            let icon = self.level_icon(&item.level);
                            let level_tag = item.level.tag().to_string();
                            let time = item.time.clone();
                            let title = item.title.clone();
                            let file_line = item.file_line.clone();
                            let body = item.body.clone();
                            let line_count = item.line_count;

                            let row_bg = if is_selected {
                                ui.visuals().selection.bg_fill
                            } else if row_num % 2 == 1 {
                                ui.visuals().faint_bg_color
                            } else {
                                Color32::TRANSPARENT
                            };

                            let row_stroke = if is_selected {
                                ui.visuals().selection.stroke
                            } else {
                                Stroke::NONE
                            };

                            let frame_resp = egui::Frame::new()
                                .fill(row_bg)
                                .stroke(row_stroke)
                                .inner_margin(egui::Margin::symmetric(6, 3))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {

                                        let expand_btn = ui.add(
                                            egui::Button::new(
                                                RichText::new(marker)
                                                    .monospace()
                                                    .color(level_color),
                                            )
                                            .frame(false),
                                        );
                                        if expand_btn.clicked() {
                                            if is_expanded {
                                                self.expanded.remove(&id);
                                            } else {
                                                self.expanded.insert(id);
                                            }
                                        }


                                        ui.label(RichText::new(icon).color(level_color));


                                        if self.show_timestamps {
                                            ui.label(
                                                RichText::new(format!("[{}]", time))
                                                    .monospace()
                                                    .size(11.5)
                                                    .color(ui.visuals().weak_text_color()),
                                            );
                                        }


                                        ui.label(
                                            RichText::new(format!("{:<5}", level_tag))
                                                .monospace()
                                                .size(11.5)
                                                .color(level_color)
                                                .strong(),
                                        );


                                        ui.label(RichText::new(&title).monospace().size(12.5));


                                        if line_count > 1 && !is_expanded {
                                            ui.weak(format!("(+{} lines)", line_count - 1));
                                        }


                                        if let Some(fl) = &file_line {
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(fl)
                                                            .monospace()
                                                            .size(11.0)
                                                            .color(ui.visuals().hyperlink_color),
                                                    );
                                                },
                                            );
                                        }
                                    });
                                });

                            let response = frame_resp.response.interact(Sense::click());


                            if response.double_clicked() {
                                if is_expanded {
                                    self.expanded.remove(&id);
                                } else {
                                    self.expanded.insert(id);
                                }
                            }


                            if response.clicked() {
                                let modifiers = ui.input(|inp| inp.modifiers);
                                if modifiers.shift {
                                    if let Some(&first) = self.selected.iter().min() {
                                        let start = first.min(id);
                                        let end = first.max(id);
                                        for idx in start..=end {
                                            self.selected.insert(idx);
                                        }
                                    } else {
                                        self.selected.insert(id);
                                    }
                                } else if modifiers.ctrl || modifiers.command {
                                    if is_selected {
                                        self.selected.remove(&id);
                                    } else {
                                        self.selected.insert(id);
                                    }
                                } else {
                                    self.selected.clear();
                                    self.selected.insert(id);
                                }
                            }


                            response.context_menu(|ui| {
                                if !self.selected.contains(&id) {
                                    self.selected.clear();
                                    self.selected.insert(id);
                                }

                                if ui.button(format!("{} Copy Entry", egui_phosphor::regular::COPY)).clicked() {
                                    copy_to_clipboard(ui.ctx(), &body);
                                    ui.close();
                                }
                                if let Some(fl) = &file_line {
                                    if ui.button(format!("{} Copy Location ({})", egui_phosphor::regular::FILE, fl)).clicked() {
                                        copy_to_clipboard(ui.ctx(), fl);
                                        ui.close();
                                    }
                                }
                                ui.separator();
                                if ui.button(format!("{} Copy All Visible", egui_phosphor::regular::BOOKS)).clicked() {
                                    let all: String = visible_indices
                                        .iter()
                                        .filter_map(|&idx| self.items.get(idx))
                                        .map(|it| it.body.clone())
                                        .collect::<Vec<_>>()
                                        .join("\n\n---\n\n");
                                    copy_to_clipboard(ui.ctx(), &all);
                                    ui.close();
                                }
                            });


                            if is_expanded {
                                egui::Frame::group(ui.style())
                                    .fill(ui.visuals().panel_fill)
                                    .inner_margin(egui::Margin::symmetric(12, 8))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.weak("Full Message:");
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.small_button(egui_phosphor::regular::COPY).clicked() {
                                                        copy_to_clipboard(ui.ctx(), &body);
                                                    }
                                                },
                                            );
                                        });
                                        ui.separator();
                                        ui.label(RichText::new(&body).monospace().size(12.0));
                                    });
                                ui.add_space(2.0);
                            }
                        }

                        if self.auto_scroll {
                            ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                        }
                    });
            });
    }

    fn draw_footer(&mut self, ui: &mut egui::Ui) {
        let visible_count = self.items.iter().filter(|it| self.should_show(it)).count();

        ui.horizontal(|ui| {
            ui.weak(format!(
                "Showing {} of {} entries",
                visible_count,
                self.items.len()
            ));

            if !self.selected.is_empty() {
                ui.separator();
                ui.label(
                    RichText::new(format!("{} selected", self.selected.len()))
                        .size(12.0)
                        .color(ui.visuals().hyperlink_color),
                );

                if ui.small_button(egui_phosphor::regular::COPY).clicked() {
                    let text = self
                        .items
                        .iter()
                        .filter(|it| self.selected.contains(&it.id))
                        .map(|it| it.body.clone())
                        .collect::<Vec<_>>()
                        .join("\n\n---\n\n");
                    copy_to_clipboard(ui.ctx(), &text);
                }

                if ui.small_button("Deselect").clicked() {
                    self.selected.clear();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Collapse All").clicked() {
                    self.expanded.clear();
                }
                if ui.small_button("Expand All").clicked() {
                    for item in &self.items {
                        if self.should_show(item) {
                            self.expanded.insert(item.id);
                        }
                    }
                }
            });
        });


        let ctx = ui.ctx().clone();
        if ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(egui::Key::C))
        {
            if !self.selected.is_empty() {
                let text = self
                    .items
                    .iter()
                    .filter(|it| self.selected.contains(&it.id))
                    .map(|it| it.body.clone())
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n");
                copy_to_clipboard(&ctx, &text);
            } else if let Some(item) = self.items.last() {
                copy_to_clipboard(&ctx, &item.body);
            }
        }
    }
}

fn extract_file_line(msg: &str) -> Option<String> {
    for line in msg.lines() {
        let dot_rs_pos = line.find(".rs:")?;
        let start = line[..dot_rs_pos].rfind('/').map(|s| s + 1).unwrap_or(0);
        let end = dot_rs_pos + 4;
        let after = &line[end..];
        let colon_pos = after.find(':').unwrap_or(after.len());
        let line_part = &after[..colon_pos];
        if !line_part.is_empty() && line_part.chars().all(|c| c.is_ascii_digit()) {
            let file = &line[start..end];
            return Some(format!("{}:{}", file, line_part));
        }
    }
    None
}

impl magic_api_editor::api::window::EditorWindow for OracleLogsWindow {
    fn id(&self) -> magic_api_editor::api::window::WindowId {
        "oracle_logs".to_string()
    }

    fn title(&self) -> &str {
        "Oracle Logs"
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        self.draw_toolbar(ui);
        ui.add_space(4.0);
        self.draw_logs(ui);
        ui.add_space(4.0);
        self.draw_footer(ui);
    }
}