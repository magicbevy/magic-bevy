use std::collections::HashSet;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::log::{global_store, LogEntry, LogLevel};
use chrono::Timelike;
use eframe::egui;

fn copy_to_clipboard(text: &str) {
    if let Ok(mut child) = Command::new("wl-copy").stdin(Stdio::piped()).spawn() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
        return;
    }
    if let Ok(mut child) = Command::new("xclip").args(["-selection", "clipboard"]).stdin(Stdio::piped()).spawn() {
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
    prev_filter_info: bool,
    prev_filter_warn: bool,
    prev_filter_error: bool,
    prev_filter_success: bool,
    prev_filter_debug: bool,
    auto_scroll: bool,
    filter_info: bool,
    filter_warn: bool,
    filter_error: bool,
    filter_success: bool,
    filter_debug: bool,
    show_timestamps: bool,
}

struct LogItem {
    time: String,
    level: LogLevel,
    title: String,
    body: String,
}

impl OracleLogsWindow {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            expanded: HashSet::new(),
            selected: HashSet::new(),
            last_store_len: 0,
            prev_filter_info: true,
            prev_filter_warn: true,
            prev_filter_error: true,
            prev_filter_success: true,
            prev_filter_debug: true,
            auto_scroll: true,
            filter_info: true,
            filter_warn: true,
            filter_error: true,
            filter_success: true,
            filter_debug: true,
            show_timestamps: true,
        }
    }

    fn refresh_from_store(&mut self) {
        let store = global_store().lock().ok();
        let entries = match store {
            Some(s) => s.entries().to_vec(),
            None => Vec::new(),
        };

        let filters_changed = self.filter_info != self.prev_filter_info
            || self.filter_warn != self.prev_filter_warn
            || self.filter_error != self.prev_filter_error
            || self.filter_success != self.prev_filter_success
            || self.filter_debug != self.prev_filter_debug;

        if entries.len() == self.last_store_len
            && !self.items.is_empty()
            && !filters_changed
        {
            return;
        }
        self.last_store_len = entries.len();
        self.prev_filter_info = self.filter_info;
        self.prev_filter_warn = self.filter_warn;
        self.prev_filter_error = self.filter_error;
        self.prev_filter_success = self.filter_success;
        self.prev_filter_debug = self.filter_debug;

        self.expanded.clear();
        self.selected.clear();
        self.items.clear();
        for entry in entries {
            if !self.should_show(&entry) {
                continue;
            }

            let time = if self.show_timestamps {
                format!(
                    "[{:02}:{:02}:{:02}]",
                    entry.timestamp.hour(),
                    entry.timestamp.minute(),
                    entry.timestamp.second()
                )
            } else {
                String::new()
            };

            let first_line = entry.message.lines().next().unwrap_or(&entry.message).to_string();
            let file_line = extract_file_line(&entry.message);
            let title = match file_line {
                Some(fl) => format!("{} {}", first_line, fl),
                None => first_line,
            };

            self.items.push(LogItem {
                time,
                level: entry.level.clone(),
                title,
                body: entry.message.clone(),
            });
        }
    }

    fn should_show(&self, entry: &LogEntry) -> bool {
        match entry.level {
            LogLevel::Info => self.filter_info,
            LogLevel::Warn => self.filter_warn,
            LogLevel::Error => self.filter_error,
            LogLevel::Success => self.filter_success,
            LogLevel::Debug => self.filter_debug,
        }
    }

    fn level_icon(&self, level: &LogLevel) -> &'static str {
        match level {
            LogLevel::Error => "E",
            LogLevel::Warn => "W",
            LogLevel::Info => "I",
            LogLevel::Success => "S",
            LogLevel::Debug => "D",
        }
    }

    fn draw_filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.checkbox(&mut self.filter_info, "INFO");
            ui.checkbox(&mut self.filter_warn, "WARN");
            ui.checkbox(&mut self.filter_error, "ERROR");
            ui.checkbox(&mut self.filter_success, "SUCCESS");
            ui.checkbox(&mut self.filter_debug, "DEBUG");
            ui.separator();
            ui.checkbox(&mut self.show_timestamps, "Timestamps");
            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
        });
    }

    fn draw_logs(&mut self, ui: &mut egui::Ui) {
        self.refresh_from_store();

        let available_height = ui.available_height() - 40.0;
        egui::ScrollArea::vertical()
            .max_height(available_height)
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for i in 0..self.items.len() {
                    let item = &self.items[i];
                    let is_expanded = self.expanded.contains(&i);
                    let is_selected = self.selected.contains(&i);
                    let marker = if is_expanded { "v" } else { ">" };
                    let level_color = item.level.color_code();
                    let icon = self.level_icon(&item.level).to_string();
                    let level_tag = item.level.tag().to_string();
                    let time = item.time.clone();
                    let title = item.title.clone();
                    let body = item.body.clone();

                    let response = ui
                        .horizontal(|ui| {
                            ui.add_space(4.0);
                            if ui.button(egui::RichText::new(marker).color(level_color)).clicked() {
                                if is_expanded {
                                    self.expanded.remove(&i);
                                } else {
                                    self.expanded.insert(i);
                                }
                            }

                            ui.label(egui::RichText::new(icon).color(level_color));

                            if !time.is_empty() {
                                ui.add_space(4.0);
                                ui.label(egui::RichText::new(time).color(egui::Color32::from_gray(150)));
                            }

                            ui.add_space(6.0);
                            ui.label(egui::RichText::new(level_tag).color(level_color).strong());

                            ui.add_space(6.0);
                            ui.label(egui::RichText::new(title));
                        })
                        .response;

                    if response.clicked() {
                        let input = ui.input(|i| i.clone());
                        if input.modifiers.shift {
                            if let Some(&first) = self.selected.iter().min() {
                                let start = first.min(i);
                                let end = first.max(i);
                                for idx in start..=end {
                                    self.selected.insert(idx);
                                }
                            } else {
                                self.selected.insert(i);
                            }
                        } else if input.modifiers.ctrl {
                            if is_selected {
                                self.selected.remove(&i);
                            } else {
                                self.selected.insert(i);
                            }
                        } else {
                            self.selected.clear();
                            self.selected.insert(i);
                        }
                    }

                    if response.secondary_clicked() {
                        self.selected.clear();
                        self.selected.insert(i);
                        response.context_menu(|ui| {
                            if ui.button("Copy").clicked() {
                                copy_to_clipboard(&body);
                                ui.close();
                            }
                            if ui.button("Copy all").clicked() {
                                let all: String = self
                                    .items
                                    .iter()
                                    .map(|it| it.body.clone())
                                    .collect::<Vec<_>>()
                                    .join("\n\n---\n\n");
                                copy_to_clipboard(&all);
                                ui.close();
                            }
                        });
                    }

                    if is_selected {
                        ui.painter().rect_filled(
                            response.rect,
                            2.0,
                            egui::Color32::from_rgb(60, 90, 140),
                        );
                    }

                    if is_expanded {
                        ui.add_space(2.0);
                        egui::Frame::new()
                            .fill(egui::Color32::from_rgb(25, 25, 35))
                            .inner_margin(8.0)
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(body)
                                        .monospace()
                                        .color(egui::Color32::from_gray(220)),
                                );
                            });
                        ui.add_space(2.0);
                    }
                }

                if self.auto_scroll {
                    ui.scroll_to_cursor(Some(egui::Align::TOP));
                }
            });
    }

    fn draw_controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Clear").clicked() {
                if let Ok(mut store) = global_store().lock() {
                    store.clear();
                }
                self.items.clear();
                self.selected.clear();
            }
            ui.separator();

            if !self.selected.is_empty() {
                if ui.button(format!("Copy {} selected", self.selected.len())).clicked() {
                    let sel: Vec<&LogItem> = self
                        .selected
                        .iter()
                        .filter_map(|i| self.items.get(*i))
                        .collect();
                    let text = sel
                        .iter()
                        .map(|it| it.body.clone())
                        .collect::<Vec<_>>()
                        .join("\n\n---\n\n");
                    copy_to_clipboard(&text);
                }
                ui.separator();
            }

            ui.label(format!("Entries: {}", self.items.len()));
            if !self.selected.is_empty() {
                ui.label(format!("Selected: {}", self.selected.len()));
            }
        });

        let ctx = ui.ctx();
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
            if !self.selected.is_empty() {
                let sel: Vec<&LogItem> = self
                    .selected
                    .iter()
                    .filter_map(|i| self.items.get(*i))
                    .collect();
                let text = sel
                    .iter()
                    .map(|it| it.body.clone())
                    .collect::<Vec<_>>()
                    .join("\n\n---\n\n");
                copy_to_clipboard(&text);
            } else if let Some(item) = self.items.last() {
                copy_to_clipboard(&item.body);
            }
        }
    }
}

fn extract_file_line(msg: &str) -> Option<String> {
    for line in msg.lines() {
        let dot_rs_pos = line.find(".rs:")?;
        let start = line[..dot_rs_pos]
            .rfind('/')
            .map(|s| s + 1)
            .unwrap_or(0);
        let end = dot_rs_pos + 4;
        let after = &line[end..];
        let colon_pos = after.find(':').unwrap_or(0);
        let line_part = &after[..colon_pos];
        if line_part.chars().all(|c| c.is_ascii_digit()) {
            let file = &line[start..end];
            return Some(format!("{}:{}", file, line_part));
        }
    }
    None
}

impl crate::api::window::EditorWindow for OracleLogsWindow {
    fn id(&self) -> crate::api::window::WindowId {
        "oracle_logs".to_string()
    }

    fn title(&self) -> &str {
        "Oracle Logs"
    }

    fn draw(&mut self, ui: &mut egui::Ui) {
        ui.heading("Oracle Logs");
        ui.separator();

        self.draw_filters(ui);
        ui.separator();

        self.draw_logs(ui);
        ui.separator();

        self.draw_controls(ui);
    }
}
