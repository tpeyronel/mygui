use glam::{Vec2, Vec4};

type Color = Vec4;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub bbox: Vec4,          // x: left, y: bottom, z: top, w: right
    pub border_radius: Vec4, // x: bottom left, y: bottom right, z: top right, w: top left (CCW)
    pub border_width: Vec4,  // x: bottom, y: right, z: top, w: left (CCW)
    pub color: Color,
    pub pos: Vec2,
}

impl Vertex {
    const VERTEX_ATTRIBUTES: &[wgpu::VertexAttribute] = &[
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: 0,
            shader_location: 0,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: std::mem::size_of::<Vec4>() as u64,
            shader_location: 1,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: (2 * std::mem::size_of::<Vec4>()) as u64,
            shader_location: 2,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x4,
            offset: (3 * std::mem::size_of::<Vec4>()) as u64,
            shader_location: 3,
        },
        wgpu::VertexAttribute {
            format: wgpu::VertexFormat::Float32x2,
            offset: (4 * std::mem::size_of::<Vec4>()) as u64,
            shader_location: 4,
        },
    ];

    pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: Self::VERTEX_ATTRIBUTES,
        }
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

pub const INDICES: &[u32] = &[0, 1, 2, 0, 2, 3];
