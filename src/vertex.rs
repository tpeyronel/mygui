use glam::Vec3;

type Color = Vec3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Color,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}


pub const INDICES: &[u32] = &[0, 1, 2, 0, 2, 3];