use super::mesh::{Mesh, MeshId};

pub struct MeshManager {
    meshes: Vec<Mesh>,
}

impl MeshManager {
    pub fn new() -> Self {
        Self { meshes: Vec::new() }
    }

    pub fn register_mesh(&mut self, mesh: Mesh) -> MeshId {
        let mesh_id = self.meshes.len();
        self.meshes.push(mesh);
        MeshId(mesh_id)
    }

    pub fn meshes(&self) -> &[Mesh] {
        &self.meshes
    }

    pub fn clear(&mut self) {
        self.meshes.clear();
    }
}
