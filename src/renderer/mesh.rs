use std::collections::HashMap;

use glam::Vec4;

use crate::{
    font::font_face::GlyphPixelMode,
    image::image_manager::ImageId,
    ui::{self, draw_element::DrawElement, mesh::VertexAttribute},
    vertex::Color,
};

pub enum Mesh {
    Mesh(ui::mesh::Mesh),
    TextGlyph {
        mesh: ui::mesh::Mesh,
        text_color: Color,
        image_id: ImageId,
        pixel_mode: GlyphPixelMode,
    },
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct RectangleData {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
    fill_color: Color,
    border_color: Color,
    border_radius: Vec4,
    border_width: Vec4,
}

impl Mesh {
    pub fn from_draw_element(draw_element: &DrawElement) -> Self {
        match draw_element {
            DrawElement::TextGlyph {
                bounds,
                uv_rectangle,
                text_color,
                image_id,
                pixel_mode,
            } => {
                let mut positions = Vec::new();
                positions.extend_from_slice(bytemuck::bytes_of(&bounds.bottom_left()));
                positions.extend_from_slice(bytemuck::bytes_of(&bounds.bottom_right()));
                positions.extend_from_slice(bytemuck::bytes_of(&bounds.top_right()));
                positions.extend_from_slice(bytemuck::bytes_of(&bounds.top_left()));

                let mut uvs = Vec::new();
                uvs.extend_from_slice(bytemuck::bytes_of(&uv_rectangle.bottom_left()));
                uvs.extend_from_slice(bytemuck::bytes_of(&uv_rectangle.bottom_right()));
                uvs.extend_from_slice(bytemuck::bytes_of(&uv_rectangle.top_right()));
                uvs.extend_from_slice(bytemuck::bytes_of(&uv_rectangle.top_left()));

                let mut vertex_attributes = HashMap::new();
                vertex_attributes.insert(VertexAttribute::Position, positions);
                vertex_attributes.insert(VertexAttribute::Uv, uvs);

                let indices = vec![0, 1, 2, 0, 2, 3];

                let mesh = ui::mesh::Mesh {
                    vertex_attributes,
                    indices,
                };

                Self::TextGlyph {
                    mesh,
                    text_color: *text_color,
                    image_id: *image_id,
                    pixel_mode: *pixel_mode,
                }
            }
            DrawElement::Mesh(mesh) => Self::Mesh(mesh.clone()),
        }
    }
}
