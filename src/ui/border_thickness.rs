use glam::{Vec2, Vec4};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BorderThickness {
    left: f32,
    bottom: f32,
    right: f32,
    top: f32,
}

impl BorderThickness {
    pub fn new(left: f32, bottom: f32, right: f32, top: f32) -> Self {
        Self {
            left: left.round(),
            bottom: bottom.round(),
            right: right.round(),
            top: top.round(),
        }
    }

    pub fn all(x: f32) -> Self {
        let x = x.round();
        Self::new(x, x, x, x)
    }

    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.left(), self.bottom(), self.right(), self.top())
    }

    pub fn delta_size(&self) -> Vec2 {
        Vec2::new(self.left() + self.right(), self.bottom() + self.top())
    }

    pub fn delta_position(&self) -> Vec2 {
        Vec2::new(self.left(), self.bottom())
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

impl Default for BorderThickness {
    fn default() -> Self {
        Self::all(0.0)
    }
}
