use bytemuck::{Pod, Zeroable};
use glam::Vec4;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Zeroable, Pod)]
pub struct Color(Vec4);

impl Color {
    pub const WHITE: Self = Self::rgba(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::rgba(0.0, 0.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::rgba(0.0, 0.0, 0.0, 0.0);
    pub const RED: Self = Self::rgba(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::rgba(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::rgba(0.0, 0.0, 1.0, 1.0);
    pub const YELLOW: Self = Self::rgba(1.0, 1.0, 0.0, 1.0);
    pub const CYAN: Self = Self::rgba(0.0, 1.0, 1.0, 1.0);
    pub const PURPLE: Self = Self::rgba(1.0, 0.0, 1.0, 1.0);

    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self(Vec4::new(r * a, g * a, b * a, a))
    }
}

impl From<Color> for Vec4 {
    fn from(value: Color) -> Self {
        value.0
    }
}
