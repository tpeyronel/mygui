use glam::{Vec2, Vec4};

use crate::vertex::{Color, Vertex};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rectangle {
    pub position: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub border_radius: Vec4,
    pub border_width: Vec4,
}

impl Rectangle {
    pub fn to_vertices(self) -> Vec<Vertex> {
        let bl = self.position;
        let br = Vec2::new(self.position.x + self.size.x, self.position.y);
        let tr = self.position + self.size;
        let tl = Vec2::new(self.position.x, self.position.y + self.size.y);

        let bbox = Vec4::new(bl.x, bl.y, tr.x, tr.y);
        let color = self.color;
        let border_radius = self.border_radius;
        let border_width = self.border_width;

        return vec![
            Vertex {
                pos: bl,
                color,
                bbox,
                border_radius,
                border_width,
                padding: Vec2::ZERO,
            },
            Vertex {
                pos: br,
                color,
                bbox,
                border_radius,
                border_width,
                padding: Vec2::ZERO,
            },
            Vertex {
                pos: tr,
                color,
                bbox,
                border_radius,
                border_width,
                padding: Vec2::ZERO,
            },
            Vertex {
                pos: tl,
                color,
                bbox,
                border_radius,
                border_width,
                padding: Vec2::ZERO,
            },
        ];
    }
}
