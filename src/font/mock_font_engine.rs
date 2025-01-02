use glam::Vec2;

use crate::{
    image::image_manager::{ImageId, ImageManager},
    rectangle::Rectangle,
};

use super::{
    font_engine::{FontEngine, LaidOutGlyph, TextLayout, TextLayoutOptions, TextMap},
    font_face::GlyphPixelMode,
};

pub struct MockFontEngine;

impl MockFontEngine {
    #[allow(unused)]
    pub fn new() -> Self {
        Self
    }
}

impl FontEngine for MockFontEngine {
    fn update(&mut self, _image_manager: &mut ImageManager) {}

    fn lay_out_text(&mut self, text: &str, options: &TextLayoutOptions) -> TextLayout {
        let mut width: f32 = 0.0;
        let mut height: f32 = 0.0;

        let mut glyphs = vec![];
        for line in text.split("\n") {
            let mut x = 0.0;

            for _c in line.chars() {
                if x + options.font_size > options.max_line_width {
                    x = 0.0;
                    height += 1.0;
                }

                let y = -(height + 1.0) * options.line_height;
                glyphs.push(LaidOutGlyph {
                    position: Vec2::new(x, y),
                    size: Vec2::splat(options.font_size),
                    atlas_uv_rectangle: Rectangle::from_position_size(Vec2::ZERO, Vec2::ONE),
                    image_id: ImageId::NULL,
                    pixel_mode: GlyphPixelMode::Grayscale,
                });

                x += options.font_size;
            }

            width = width.max(line.chars().count() as f32);
            height += 1.0;
        }

        width *= options.font_size;
        height *= options.line_height;

        let size = Vec2::new(width, height);
        let text_map = TextMap::new();

        TextLayout { size, glyphs, text_map }
    }

    fn on_exit(&mut self, _: &ImageManager) {}
}
