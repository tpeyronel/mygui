use std::collections::HashMap;

use glam::{Vec2, Vec4};

use super::mesh::{Mesh, VertexAttribute};

pub struct ColorMeshBuilder {
    positions: Vec<u8>,
    colors: Vec<u8>,
    indices: Vec<[u32; 3]>,
}

impl ColorMeshBuilder {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, position: Vec2, color: Vec4) -> u32 {
        let vertex_index = (self.positions.len() / std::mem::size_of::<Vec2>()) as u32;
        self.positions.extend_from_slice(bytemuck::bytes_of(&position));
        self.colors.extend_from_slice(bytemuck::bytes_of(&color));
        vertex_index
    }

    pub fn add_triangle(&mut self, i: u32, j: u32, k: u32) {
        self.indices.push([i, j, k]);
    }

    pub fn add_quad(&mut self, i: u32, j: u32, k: u32, l: u32) {
        self.indices.push([i, j, k]);
        self.indices.push([i, k, l]);
    }

    pub fn build(self) -> Mesh {
        let mut vertex_attributes: HashMap<VertexAttribute, Vec<u8>> = HashMap::new();
        vertex_attributes.insert(VertexAttribute::Position, self.positions);
        vertex_attributes.insert(VertexAttribute::Color, self.colors);

        let indices = self.indices.concat();

        Mesh {
            vertex_attributes,
            indices,
        }
    }
}
