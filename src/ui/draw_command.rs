use crate::mesh::mesh::MeshId;

#[derive(Debug, Clone, Copy)]
pub enum DrawCommand {
    BindShader(Shader),
    DrawMesh(MeshId),
    SetStencilReference(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shader {
    Shape,
    ShapeClip,
    ShapeClipRevert,
    Texture,
    TextGrayscale,
    TextSubpixel,
}
