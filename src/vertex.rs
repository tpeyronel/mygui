use glam::{Vec2, Vec4};

type Color = Vec4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub color: Color,
    pub pos: Vec2,
    pub bbox_bottom_left: Vec2,
    pub bbox_top_right: Vec2,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}


pub const INDICES: &[u32] = &[0, 1, 2, 0, 2, 3];