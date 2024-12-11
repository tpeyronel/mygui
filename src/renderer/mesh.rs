use glam::{Vec2, Vec4};

use crate::{
    image::image_manager::ImageId,
    rectangle::Rectangle,
    ui::draw_element::DrawElement,
    vertex::{Color, Vertex},
};

pub enum Mesh {
    Rectangle {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        rectangle_data: RectangleData,
    },
    Texture {
        vertices: Vec<Vertex>,
        indices: Vec<u32>,
        image_id: ImageId,
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
            DrawElement::Texture {
                bounds,
                uv_rectangle,
                image_id,
            } => {
                let vertices = rectangle_to_vertices(bounds, uv_rectangle);
                let indices = vec![0, 1, 2, 0, 2, 3];

                Self::Texture {
                    vertices,
                    indices,
                    image_id: *image_id,
                }
            }
        }
    }
}

fn rectangle_to_vertices(rectangle: &Rectangle, uv_rectangle: &Rectangle) -> Vec<Vertex> {
    let bl = Vec2::new(rectangle.left, rectangle.bottom);
    let br = Vec2::new(rectangle.right, rectangle.bottom);
    let tr = Vec2::new(rectangle.right, rectangle.top);
    let tl = Vec2::new(rectangle.left, rectangle.top);

    let uv_bl = Vec2::new(uv_rectangle.left, uv_rectangle.bottom);
    let uv_br = Vec2::new(uv_rectangle.right, uv_rectangle.bottom);
    let uv_tr = Vec2::new(uv_rectangle.right, uv_rectangle.top);
    let uv_tl = Vec2::new(uv_rectangle.left, uv_rectangle.top);

    return vec![
        Vertex { pos: bl, uv: uv_bl },
        Vertex { pos: br, uv: uv_br },
        Vertex { pos: tr, uv: uv_tr },
        Vertex { pos: tl, uv: uv_tl },
    ];
}
