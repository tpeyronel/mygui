use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Padding {
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
    pub left: f32,
}

impl Padding {
    #[allow(unused)]
    pub fn new(bottom: f32, right: f32, top: f32, left: f32) -> Self {
        Self {
            bottom,
            right,
            top,
            left,
        }
    }

    pub fn all(x: f32) -> Self {
        Self {
            bottom: x,
            right: x,
            top: x,
            left: x,
        }
    }

    pub fn delta_size(&self) -> Vec2 {
        Vec2::new(
            self.left.round() + self.right.round(),
            self.bottom.round() + self.top.round(),
        )
    }

    pub fn delta_position(&self) -> Vec2 {
        Vec2::new(self.left.round(), self.bottom.round())
    }
}

impl Default for Padding {
    fn default() -> Self {
        Self {
            bottom: 0.0,
            right: 0.0,
            top: 0.0,
            left: 0.0,
        }
    }
}
