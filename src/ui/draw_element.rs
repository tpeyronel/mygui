use glam::Vec4;

use crate::{
    font::font_face::GlyphPixelMode,
    image::image_manager::ImageId,
    rectangle::Rectangle,
    vertex::{Color, Vertex},
};

#[derive(Debug, Clone, PartialEq)]
pub enum DrawElement {
    Rectangle {
        bounds: Rectangle,
        fill_color: Color,
        border_color: Color,
        border_radius: Vec4,
        border_width: Vec4,
    },
    TextGlyph {
        bounds: Rectangle,
        uv_rectangle: Rectangle,
        text_color: Color,
        image_id: ImageId,
        pixel_mode: GlyphPixelMode,
    },
    Mesh {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        fill_color: Color,
    },
}
