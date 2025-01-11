use crate::{color::Color, font::font_face::GlyphPixelMode, image::image_manager::ImageId, rectangle::Rectangle};

use super::mesh::Mesh;

#[derive(Debug, Clone, PartialEq)]
pub enum DrawElement {
    TextGlyph {
        bounds: Rectangle,
        uv_rectangle: Rectangle,
        text_color: Color,
        image_id: ImageId,
        pixel_mode: GlyphPixelMode,
    },
    Mesh(Mesh),
}
