use magic_api_editor::{Color, Rect, Ui};

pub struct EguiBridge {
    inner: *mut egui::Ui,
}

impl EguiBridge {
    pub fn new(ui: &mut egui::Ui) -> Self {
        Self {
            inner: ui as *mut egui::Ui,
        }
    }

    pub fn get(&self) -> &mut egui::Ui {
        unsafe { &mut *self.inner }
    }
}

impl Ui for EguiBridge {
    fn as_any(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn label(&mut self, text: &str) {
        self.get().label(text);
    }

    fn button(&mut self, text: &str) -> bool {
        self.get().button(text).clicked()
    }

    fn separator(&mut self) {
        self.get().separator();
    }
}

pub fn downcast_egui(ui: &mut dyn Ui) -> &mut egui::Ui {
    ui.as_any()
        .downcast_mut::<EguiBridge>()
        .expect("EguiBridge required: window must be drawn by the egui host")
        .get()
}

pub trait ColorExt {
    fn to_egui(self) -> egui::Color32;
}

impl ColorExt for Color {
    fn to_egui(self) -> egui::Color32 {
        egui::Color32::from_rgba_unmultiplied(self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

pub trait RectExt {
    fn to_egui(self) -> egui::Rect;
}

impl RectExt for Rect {
    fn to_egui(self) -> egui::Rect {
        egui::Rect::from_min_max(
            egui::pos2(self.0[0], self.0[1]),
            egui::pos2(self.0[2], self.0[3]),
        )
    }
}