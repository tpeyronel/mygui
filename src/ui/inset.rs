use glam::Vec2;

use super::extent::{ExtentExt, ExtrinsicExtent};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Inset {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

impl Inset {
    pub fn new(left: f32, bottom: f32, right: f32, top: f32) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
        }
    }

    pub fn all(x: f32) -> Self {
        Self::new(x, x, x, x)
    }

    pub fn delta_size(&self) -> Vec2 {
        Vec2::new(self.left + self.right, self.bottom + self.top)
    }

    pub fn delta_position(&self) -> Vec2 {
        Vec2::new(self.left, self.bottom)
    }

    pub fn left(&self) -> f32 {
        self.left
    }

    pub fn bottom(&self) -> f32 {
        self.bottom
    }

    pub fn right(&self) -> f32 {
        self.right
    }

    pub fn top(&self) -> f32 {
        self.top
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExtrinsicInset {
    left: ExtrinsicExtent,
    bottom: ExtrinsicExtent,
    right: ExtrinsicExtent,
    top: ExtrinsicExtent,
}

impl ExtrinsicInset {
    pub fn new(left: ExtrinsicExtent, bottom: ExtrinsicExtent, right: ExtrinsicExtent, top: ExtrinsicExtent) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
        }
    }

    pub fn all(x: ExtrinsicExtent) -> Self {
        Self::new(x, x, x, x)
    }

    pub fn hor_ver(horizontal: ExtrinsicExtent, vertical: ExtrinsicExtent) -> Self {
        Self::new(horizontal, vertical, horizontal, vertical)
    }

    pub fn resolve(&self, boundary_size: Vec2, dp_factor: f32) -> Inset {
        Inset::new(
            self.left.resolve(boundary_size.x, 0.0, dp_factor),
            self.bottom.resolve(boundary_size.y, 0.0, dp_factor),
            self.right.resolve(boundary_size.x, 0.0, dp_factor),
            self.top.resolve(boundary_size.y, 0.0, dp_factor),
        )
    }
}

impl Default for ExtrinsicInset {
    fn default() -> Self {
        Self::all(0.px())
    }
}

impl From<ExtrinsicExtent> for ExtrinsicInset {
    fn from(value: ExtrinsicExtent) -> Self {
        Self::all(value)
    }
}
