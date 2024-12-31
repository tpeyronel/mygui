use glam::Vec2;

use crate::{
    image::image_manager::{ImageId, ImageManager},
    rectangle::Rectangle,
};

use super::font_face::GlyphPixelMode;

pub trait FontEngine {
    fn update(&mut self, image_manager: &mut ImageManager);
    fn lay_out_text(&mut self, text: &str, options: &TextLayoutOptions) -> (Vec2, Vec<LaidOutGlyph>);
}

pub struct TextLayoutOptions<'a> {
    pub font_family: &'a str,
    pub font_size: f32,
    pub line_height: f32,
    pub max_line_width: f32,
}

pub struct LaidOutGlyph {
    pub position: Vec2, // Bottom left of the glyph's bounding box
    pub size: Vec2,     // Size of the glyph's bounding box
    pub atlas_uv_rectangle: Rectangle,
    pub image_id: ImageId,
    pub pixel_mode: GlyphPixelMode,
}
