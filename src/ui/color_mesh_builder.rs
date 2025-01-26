use glam::Vec2;

use crate::{
    color::Color,
    mesh::mesh::{Mesh, VertexAttribute},
};

use super::mesh_builder::MeshBuilder;

pub struct ColorMeshBuilder {
    builder: MeshBuilder<2>,
}

impl ColorMeshBuilder {
    pub fn new() -> Self {
        Self {
            builder: MeshBuilder::new(),
        }
    }

    pub fn add_vertex(&mut self, position: Vec2, color: &Color) -> u32 {
        self.builder
            .add_vertex([bytemuck::bytes_of(&position), bytemuck::bytes_of(color)])
    }

    pub fn add_triangle(&mut self, i: u32, j: u32, k: u32) {
        self.builder.add_triangle(i, j, k)
    }

    pub fn add_quad(&mut self, i: u32, j: u32, k: u32, l: u32) {
        self.builder.add_quad(i, j, k, l)
    }

    pub fn build(self) -> Mesh {
        self.builder.build([VertexAttribute::Position, VertexAttribute::Color])
    }
}
