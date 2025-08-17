// use super::hal;
use bevy::prelude::*;
use embedded_graphics::{pixelcolor::Rgb565, prelude::Point};
use picocalc_bevy::Visible;

#[derive(Component, Default)]
#[require(Visible)]
pub struct Renderable;

#[derive(Component, Default)]
#[require(Renderable)]
pub struct TextComponent {
    pub text: String,
    pub old: Option<String>,
    pub point: Point,
    pub bg_color: Option<Rgb565>,
    pub color: Option<Rgb565>,
}

impl TextComponent {
    pub fn set_text(&mut self, text: impl ToString) {
        self.old = Some(self.text.clone());
        self.text = text.to_string();
    }

    pub fn was_rendered(&mut self) {
        self.old = None;
    }
}

#[derive(Component)]
#[require(Renderable)]
pub struct Shape {
    // pub vertices: Vec<[f32; 3]>,
    // pub lines: Vec<[usize; 2]>,
    // pub faces: Vec<[usize; 3]>,
    // pub render_mode: RenderMode,
    // pub scale: f32,
    // pub color: Rgb565,
}

impl Default for Shape {
    fn default() -> Self {
        Self {
            // vertices: Vec::new(),
            // lines: Vec::new(),
            // faces: Vec::new(),
            // render_mode: RenderMode::Lines,
            // scale: 1.0,
            // color: Rgb565::GREEN,
        }
    }
}
