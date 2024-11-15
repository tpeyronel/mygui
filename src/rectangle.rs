use glam::{Vec2, Vec4};

use crate::vertex::{Color, Vertex};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Rectangle {
    pub position: Vec2,
    pub size: Vec2,
    pub fill_color: Color,
    pub border_color: Color,
    pub border_radius: Vec4,
    pub border_width: Vec4,
}

impl Rectangle {
    pub fn to_vertices(self) -> Vec<Vertex> {
        let bl = self.position;
        let br = Vec2::new(self.position.x + self.size.x, self.position.y);
        let tr = self.position + self.size;
        let tl = Vec2::new(self.position.x, self.position.y + self.size.y);

        return vec![
            Vertex { pos: bl },
            Vertex { pos: br },
            Vertex { pos: tr },
            Vertex { pos: tl },
        ];
    }
}
