use std::u32;

use crate::{mesh::mesh::MeshId, rectangle::Rectangle};

#[derive(Debug, Clone, Copy)]
pub enum DrawCommand {
    BindShader(Shader),
    DrawMesh(MeshId),
    SetStencilReference(u32),
    SetScissor(ScissorRectangle),
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

#[derive(Debug, Clone, Copy)]
pub struct ScissorRectangle {
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
    pub top: u32,
}

impl ScissorRectangle {
    pub const NO_SCISSOR: Self = Self {
        left: u32::MIN,
        bottom: u32::MIN,
        right: u32::MAX,
        top: u32::MAX,
    };

    pub fn from_rectangle(rectangle: &Rectangle) -> Self {
        Self {
            left: rectangle.left.floor() as u32,
            bottom: rectangle.bottom.floor() as u32,
            right: rectangle.right.ceil() as u32,
            top: rectangle.top.ceil() as u32,
        }
    }

    pub fn intersect(&self, other: &ScissorRectangle) -> ScissorRectangle {
        ScissorRectangle {
            left: self.left.max(other.left),
            bottom: self.bottom.max(other.bottom),
            right: self.right.min(other.right),
            top: self.top.min(other.top),
        }
    }

    pub fn x(&self) -> u32 {
        self.left
    }

    pub fn y(&self) -> u32 {
        self.bottom
    }

    pub fn width(&self) -> u32 {
        self.right.saturating_sub(self.left)
    }

    pub fn height(&self) -> u32 {
        self.top.saturating_sub(self.bottom)
    }

    pub fn to_x_y_width_height_clamp(&self, framebuffer_width: u32, framebuffer_height: u32) -> (u32, u32, u32, u32) {
        let x = self.x();
        let y = self.y();
        let width = self.width();
        let height = self.height();

        // Flip Y coordinate, as scissor (0, 0) is top-left.
        let y = framebuffer_height.saturating_sub(y).saturating_sub(height);

        let x = x.min(framebuffer_width);

        let max_width = framebuffer_width - x;
        let max_height = framebuffer_height - y;
        let width = width.min(max_width);
        let height = height.min(max_height);

        (x, y, width, height)
    }
}
