use glam::Vec2;

use crate::{
    image::image_manager::{ImageId, ImageManager},
    rectangle::Rectangle,
    text::text_position::TextPosition,
};

use super::font_face::GlyphPixelMode;

pub trait FontEngine {
    fn update(&mut self, image_manager: &mut ImageManager);
    fn lay_out_text(&mut self, text: &str, options: &TextLayoutOptions) -> TextLayout;
    fn on_exit(&mut self, image_manager: &ImageManager);
}

pub struct TextLayoutOptions<'a> {
    pub font_family: &'a str,
    pub font_size: f32,
    pub line_height: f32,
    pub max_line_width: f32,
}

pub struct TextLayout {
    pub size: Vec2,
    pub glyphs: Vec<LaidOutGlyph>,
    pub text_map: TextMap,
}

pub struct LaidOutGlyph {
    pub position: Vec2, // Bottom left of the glyph's bounding box
    pub size: Vec2,     // Size of the glyph's bounding box
    pub atlas_uv_rectangle: Rectangle,
    pub image_id: ImageId,
    pub pixel_mode: GlyphPixelMode,
}

struct TextMapLine {
    y: f32,
    columns: Vec<f32>,
}

pub struct TextMap {
    lines: Vec<TextMapLine>, // lines[i].columns[j] corresponds with TextPosition { line: i, column: j }.
}

impl TextMap {
    pub fn new() -> Self {
        Self { lines: Vec::new() }
    }

    pub fn insert(&mut self, text_position: TextPosition, screen_position: Vec2) {
        if text_position.column == 0 {
            assert_eq!(self.lines.len() as u32, text_position.line);
            self.lines.push(TextMapLine {
                y: screen_position.y,
                columns: vec![screen_position.x],
            });
        } else {
            let line = &mut self.lines[text_position.line as usize];
            assert_eq!(line.columns.len() as u32, text_position.column);
            assert_eq!(line.y, screen_position.y);
            line.columns.push(screen_position.x);
        }
    }

    pub fn get_clamped(&self, text_position: TextPosition) -> Vec2 {
        let line = self.lines.get(text_position.line as usize).or(self.lines.last());

        match line {
            Some(line) => {
                let y = line.y;
                let x = *line
                    .columns
                    .get(text_position.column as usize)
                    .unwrap_or(line.columns.last().unwrap()); // line.columns should always have at least 1 element.

                Vec2::new(x, y)
            }
            None => Vec2::ZERO,
        }
    }
}
