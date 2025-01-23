use crate::mesh::mesh::MeshId;

#[derive(Debug, Clone, Copy)]
pub enum DrawCommand {
    BindShader(Shader),
    DrawMesh(MeshId),
}

#[derive(Debug, Clone, Copy)]
pub enum Shader {
    Shape,
    ShapeClip,
    Texture,
    TextGrayscale,
    TextSubpixel,
}
