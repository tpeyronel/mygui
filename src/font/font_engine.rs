use std::{collections::HashMap, ffi::OsStr};

use glam::Vec2;

use crate::{
    image::image_manager::{ImageId, ImageManager},
    rectangle::Rectangle,
};

use super::font_atlas::{AtlasGlyph, FontAtlas};

pub struct FontEngine {
    ft_lib: freetype::Library,
    font_dir_path: String,
    fonts: HashMap<FontKey, FontData>,
}

impl FontEngine {
    pub fn new(font_dir_path: impl AsRef<OsStr>) -> Self {
        let ft_lib = freetype::Library::init().unwrap();
        ft_lib.set_lcd_filter(freetype::LcdFilter::LcdFilterDefault).expect("TODO");

        Self {
            ft_lib,
            font_dir_path: font_dir_path.as_ref().to_string_lossy().into_owned(),
            fonts: HashMap::new(),
        }
    }

    pub fn lay_out_text(
        &mut self,
        image_manager: &mut ImageManager,
        text: &str,
        options: &TextLayoutOptions,
        mut f: impl FnMut(&LaidOutGlyph),
    ) -> Vec2 {
        let font_path = self.font_dir_path.clone() + options.font;
        let FontData { face, atlas } =
            self.get_or_create_font_data(image_manager, &font_path, options.font_size as u32);

        let mut pen = Vec2::ZERO;
        let mut max_computed_line_width: f32 = 0.0;

        let atlas_image = image_manager.get_image(atlas.image_id());
        let atlas_width = atlas_image.width() as f32;
        let atlas_height = atlas_image.height() as f32;

        for c in text.chars() {
            if c == '\n' {
                max_computed_line_width = max_computed_line_width.max(pen.x);
                pen.x = 0.0;
                pen.y -= options.line_height;
                continue;
            }

            let glyph_index = face.get_char_index(c as usize).expect("TODO");
            let glyph: &AtlasGlyph = atlas.get_glyph(glyph_index);

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

            let atlas_uv_rectangle = Rectangle {
                left: glyph.left as f32 / atlas_width,
                bottom: glyph.bottom as f32 / atlas_height,
                right: glyph.right as f32 / atlas_width,
                top: glyph.top as f32 / atlas_height,
            };

            let laid_out_glyph = LaidOutGlyph {
                position,
                size,
                atlas_uv_rectangle,
                image_id: atlas.image_id(),
            };

            f(&laid_out_glyph);

            pen.x += glyph.advance as f32;
        }

        max_computed_line_width = max_computed_line_width.max(pen.x);

        let dimensions = Vec2::new(
            max_computed_line_width,
            pen.y.abs() + options.line_height - Self::estimate_descender(face, options.font_size),
        );

        dimensions
    }

    // Not quite perfect (the round() doesn't always work), but good enough.
    fn estimate_descender(face: &freetype::Face, font_size: f32) -> f32 {
        let unscaled_descender = face.descender() as f32 / 64.0;
        let unscaled_height = (face.ascender() - face.descender()) as f32 / 64.0;
        let height_quotient = font_size / unscaled_height;
        let scaled_descender = (height_quotient * unscaled_descender).round();

        scaled_descender
    }

    fn get_or_create_font_data(&mut self, image_manager: &mut ImageManager, font: &str, font_size: u32) -> &FontData {
        self.fonts
            .entry(FontKey {
                path: font.to_string(),
                font_size,
            })
            .or_insert_with(|| {
                let face = self.ft_lib.new_face(font, 0).unwrap();
                face.set_pixel_sizes(0, font_size).expect("TODO");
                let atlas = FontAtlas::new(&face, image_manager);

                FontData { face, atlas }
            })
    }
}

pub struct TextLayoutOptions<'a> {
    pub font: &'a str,
    pub font_size: f32,
    pub line_height: f32,
    pub max_line_width: f32,
}

pub struct LaidOutGlyph {
    pub position: Vec2, // Bottom left of the glyph's bounding box
    pub size: Vec2,     // Size of the glyph's bounding box
    pub atlas_uv_rectangle: Rectangle,
    pub image_id: ImageId,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct FontKey {
    path: String,
    font_size: u32,
}

#[derive(Debug)]
pub struct FontData {
    face: freetype::Face,
    atlas: FontAtlas,
}
