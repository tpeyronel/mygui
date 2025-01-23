use std::collections::HashMap;

use glam::{Vec2, Vec4};

use crate::image::image_manager::ImageId;

#[derive(Debug, Clone, Copy)]
pub struct MeshId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertex_count: u32,
    pub vertex_attributes: HashMap<VertexAttribute, Vec<u8>>,
    pub indices: Vec<u32>,
    pub image_id: Option<ImageId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttribute {
    Position,
    Color,
    Uv,
}

impl VertexAttribute {
    pub fn size(&self) -> u64 {
        match self {
            VertexAttribute::Position => std::mem::size_of::<Vec2>() as u64,
            VertexAttribute::Color => std::mem::size_of::<Vec4>() as u64,
            VertexAttribute::Uv => std::mem::size_of::<Vec2>() as u64,
        }
    }
}
