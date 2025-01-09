use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub vertex_attributes: HashMap<VertexAttribute, Vec<u8>>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexAttribute {
    Position,
    Color,
    Uv,
}
