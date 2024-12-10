use std::ffi::OsStr;

use glam::Vec2;

use super::font_atlas::{AtlasGlyph, FontAtlas};

pub struct FontEngine {
    pub atlas: FontAtlas,
}

impl FontEngine {
    pub fn new(font_path: impl AsRef<OsStr>) -> Self {
        // TODO: ft lib as parameter?
        let ft_lib = freetype::Library::init().unwrap();

        Self {
            // TODO: choose font size
            atlas: FontAtlas::new(font_path, 24, &ft_lib),
        }
    }

    pub fn lay_out_text(&self, text: &str, options: &TextLayoutOptions, mut f: impl FnMut(&LaidOutGlyph)) -> Vec2 {
        let mut pen = Vec2::ZERO;
        let mut max_computed_line_width: f32 = 0.0;

        for c in text.chars() {
            if c == '\n' {
                max_computed_line_width = max_computed_line_width.max(pen.x);
                pen.x = 0.0;
                pen.y -= options.line_height;
                continue;
            }

            let glyph: &AtlasGlyph = self.atlas.get_glyph(c).expect("TODO");

            if pen.x + glyph.advance as f32 > options.max_line_width {
                max_computed_line_width = max_computed_line_width.max(pen.x);
                pen.x = 0.0;
                pen.y -= options.line_height;
            }

            let position = pen
                + Vec2::new(
                    glyph.bearing_left as f32,
                    glyph.bearing_bottom as f32 - options.line_height,
                );
            let size = Vec2::new(
                (glyph.bearing_right - glyph.bearing_left) as f32,
                (glyph.bearing_top - glyph.bearing_bottom) as f32,
            );

            let atlas_uv_bl = Vec2::new(
                glyph.left as f32 / self.atlas.image().width() as f32,
                glyph.bottom as f32 / self.atlas.image().height() as f32,
            );

            let atlas_uv_tr = Vec2::new(
                glyph.right as f32 / self.atlas.image().width() as f32,
                glyph.top as f32 / self.atlas.image().height() as f32,
            );

            let laid_out_glyph = LaidOutGlyph {
                position,
                size,
                atlas_uv_bl,
                atlas_uv_tr,
            };

            f(&laid_out_glyph);

            pen.x += glyph.advance as f32;
        }

        let dimensions = Vec2::new(
            max_computed_line_width,
            pen.y.abs() + options.line_height - (self.atlas.face.descender() / 64) as f32,
        );

        dimensions
    }
}

pub struct TextLayoutOptions {
    pub font_size: f32,
    pub line_height: f32,
    pub max_line_width: f32,
}

pub struct LaidOutGlyph {
    pub position: Vec2, // Bottom left of the glyph's bounding box
    pub size: Vec2,     // Size of the glyph's bounding box
    pub atlas_uv_bl: Vec2,
    pub atlas_uv_tr: Vec2,
}
