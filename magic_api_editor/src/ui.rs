use std::any::Any;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub [u8; 4]);

impl Color {
    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Self([r, g, b, 255])
    }

    pub const fn from_rgba_unmultiplied(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self([r, g, b, a])
    }

    pub const fn to_array(self) -> [u8; 4] {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect(pub [f32; 4]);

impl Rect {
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self([min_x, min_y, max_x, max_y])
    }

    pub const fn to_array(self) -> [f32; 4] {
        self.0
    }
}

pub trait Ui {
    fn as_any(&mut self) -> &mut dyn Any;
    fn label(&mut self, text: &str);
    fn button(&mut self, text: &str) -> bool;
    fn separator(&mut self);
}