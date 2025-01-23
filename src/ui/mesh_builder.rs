use std::collections::HashMap;

use crate::mesh::mesh::{Mesh, VertexAttribute};

pub struct MeshBuilder<const N: usize> {
    vertex_count: u32,
    attributes: [Vec<u8>; N],
    indices: Vec<[u32; 3]>,
}

impl<const N: usize> MeshBuilder<N> {
    pub fn new() -> Self {
        Self {
            vertex_count: 0,
            attributes: std::array::from_fn(|_| Vec::new()),
            indices: Vec::new(),
        }
    }

    pub fn add_vertex(&mut self, attributes: [&[u8]; N]) -> u32 {
        for (attrib, &extend) in std::iter::zip(self.attributes.iter_mut(), attributes.iter()) {
            attrib.extend(extend);
        }

        let vertex_id = self.vertex_count;
        self.vertex_count += 1;
        vertex_id
    }

    pub fn add_triangle(&mut self, i: u32, j: u32, k: u32) {
        self.indices.push([i, j, k]);
    }

    pub fn add_quad(&mut self, i: u32, j: u32, k: u32, l: u32) {
        self.indices.push([i, j, k]);
        self.indices.push([i, k, l]);
    }

    pub fn build(self, attribute_types: [VertexAttribute; N]) -> Mesh {
        let mut vertex_attributes: HashMap<VertexAttribute, Vec<u8>> = HashMap::new();

        for (data, &attrib) in std::iter::zip(self.attributes.into_iter(), attribute_types.iter()) {
            debug_assert!(!vertex_attributes.contains_key(&attrib));
            debug_assert_eq!(data.len() as u64, self.vertex_count as u64 * attrib.size());
            vertex_attributes.insert(attrib, data);
        }

        let indices = self.indices.concat();

        Mesh {
            vertex_count: self.vertex_count,
            vertex_attributes,
            indices,
            image_id: None,
        }
    }
}
