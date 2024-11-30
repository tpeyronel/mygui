use glam::{Vec2, Vec4};

#[derive(Debug, Clone, Copy)]
pub struct BorderThickness {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl BorderThickness {
    pub fn new(left: f32, bottom: f32, right: f32, top: f32) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
        }
    }

    pub fn all(x: f32) -> Self {
        Self {
            left: x,
            bottom: x,
            right: x,
            top: x,
        }
    }

    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(
            self.left.round(),
            self.bottom.round(),
            self.right.round(),
            self.top.round(),
        )
    }

    pub fn delta_size(&self) -> Vec2 {
        Vec2::new(
            self.left.round() + self.right.round(),
            self.bottom.round() + self.top.round(),
        )
    }

    pub fn delta_position(&self) -> Vec2 {
        Vec2::new(self.left.round(), self.bottom.round()).round()
    }
}

impl Default for BorderThickness {
    fn default() -> Self {
        Self {
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
            left: 0.0,
        }
    }
}
