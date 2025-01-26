use glam::Vec4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerRadius {
    bottom_left: f32,
    bottom_right: f32,
    top_right: f32,
    top_left: f32,
}

impl CornerRadius {
    pub fn new(bottom_left: f32, bottom_right: f32, top_right: f32, top_left: f32) -> Self {
        Self {
            bottom_left: bottom_left.round(),
            bottom_right: bottom_right.round(),
            top_right: top_right.round(),
            top_left: top_left.round(),
        }
    }

    pub fn all(x: f32) -> Self {
        let x = x.round();
        Self::new(x, x, x, x)
    }

    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.bottom_left, self.bottom_right, self.top_right, self.top_left)
    }

    pub fn bottom_left(&self) -> f32 {
        self.bottom_left
    }

    pub fn bottom_right(&self) -> f32 {
        self.bottom_right
    }

    pub fn top_right(&self) -> f32 {
        self.top_right
    }

    pub fn top_left(&self) -> f32 {
        self.top_left
    }
}

impl Default for CornerRadius {
    fn default() -> Self {
        Self::all(0.0)
    }
}
