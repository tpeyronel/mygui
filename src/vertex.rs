use glam::Vec3;

type Color = Vec3;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pos: Vec3,
    color: Color,
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub const VERTICES: &[Vertex] = &[
    Vertex {
        pos: Vec3::new(0.0, 1.0, 0.0),
        color: Vec3::new(1.0, 0.0, 0.0),
    },
    Vertex {
        pos: Vec3::new(-1.0, -1.0, 0.0),
        color: Vec3::new(0.0, 1.0, 0.0),
    },
    Vertex {
        pos: Vec3::new(1.0, -1.0, 0.0),
        color: Vec3::new(0.0, 0.0, 1.0),
    },
];
