use glam::Vec4;

use crate::{
    font::font_face::GlyphPixelMode,
    image::image_manager::ImageId,
    rectangle::Rectangle,
    ui::draw_element::DrawElement,
    vertex::{Color, Vertex},
};

pub enum Mesh {
    Mesh {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        rectangle_data: RectangleData,
    },
    Rectangle {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        rectangle_data: RectangleData,
    },
    TextGlyph {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
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
            DrawElement::Rectangle {
                bounds,
                fill_color,
                border_color,
                border_radius,
                border_width,
            } => {
                let vertices = rectangle_to_vertices(
                    bounds,
                    &Rectangle {
                        left: 0.0,
                        bottom: 0.0,
                        right: 0.0,
                        top: 0.0,
                    },
                );
                let indices = vec![0, 1, 2, 0, 2, 3];

                let rectangle_data = RectangleData {
                    left: bounds.left,
                    bottom: bounds.bottom,
                    right: bounds.right,
                    top: bounds.top,
                    fill_color: *fill_color,
                    border_color: *border_color,
                    border_radius: *border_radius,
                    border_width: *border_width,
                };

                Self::Rectangle {
                    vertices,
                    indices,
                    rectangle_data,
                }
            }
            DrawElement::TextGlyph {
                bounds,
                uv_rectangle,
                text_color,
                image_id,
                pixel_mode,
            } => {
                let vertices = rectangle_to_vertices(bounds, uv_rectangle);
                let indices = vec![0, 1, 2, 0, 2, 3];

                Self::TextGlyph {
                    vertices,
                    indices,
                    text_color: *text_color,
                    image_id: *image_id,
                    pixel_mode: *pixel_mode,
                }
            }
            DrawElement::Mesh {
                vertices,
                indices,
                fill_color,
            } => Self::Mesh {
                vertices: vertices.clone(),
                indices: indices.clone(),
                rectangle_data: RectangleData {
                    left: 0.0,
                    bottom: 0.0,
                    right: 0.0,
                    top: 0.0,
                    fill_color: *fill_color,
                    border_color: Color::new(1.0, 1.0, 1.0, 1.0),
                    border_radius: Vec4::ZERO,
                    border_width: Vec4::ZERO,
                },
            },
        }
    }
}

fn rectangle_to_vertices(rectangle: &Rectangle, uv_rectangle: &Rectangle) -> Vec<Vertex> {
    return vec![
        Vertex {
            pos: rectangle.bottom_left(),
            uv: uv_rectangle.bottom_left(),
        },
        Vertex {
            pos: rectangle.bottom_right(),
            uv: uv_rectangle.bottom_right(),
        },
        Vertex {
            pos: rectangle.top_right(),
            uv: uv_rectangle.top_right(),
        },
        Vertex {
            pos: rectangle.top_left(),
            uv: uv_rectangle.top_left(),
        },
    ];
}
