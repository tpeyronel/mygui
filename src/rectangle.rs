use glam::{Vec2, Vec4};

use crate::vertex::{Color, Vertex};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rectangle {
    pub position: Vec2,
    pub size: Vec2,
    pub fill_color: Color,
    pub border_color: Color,
    pub border_radius: Vec4,
    pub border_width: Vec4,
    pub uv_bl: Vec2,
    pub uv_tr: Vec2,
}

impl Rectangle {
    pub fn to_vertices(self) -> Vec<Vertex> {
        let bl = self.position;
        let br = Vec2::new(self.position.x + self.size.x, self.position.y);
        let tr = self.position + self.size;
        let tl = Vec2::new(self.position.x, self.position.y + self.size.y);

        return vec![
            Vertex {
                pos: bl,
                uv: self.uv_bl,
            },
            Vertex {
                pos: br,
                uv: Vec2::new(self.uv_tr.x, self.uv_bl.y),
            },
            Vertex {
                pos: tr,
                uv: self.uv_tr,
            },
            Vertex {
                pos: tl,
                uv: Vec2::new(self.uv_bl.x, self.uv_tr.y),
            },
        ];
    }
}
