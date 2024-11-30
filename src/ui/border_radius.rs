use glam::Vec4;

#[derive(Debug, Clone, Copy)]
pub struct BorderRadius {
    pub bottom_left: f32,
    pub bottom_right: f32,
    pub top_right: f32,
    pub top_left: f32,
}

impl BorderRadius {
    pub fn new(bottom_left: f32, bottom_right: f32, top_right: f32, top_left: f32) -> Self {
        Self {
            bottom_left,
            bottom_right,
            top_right,
            top_left,
        }
    }

    pub fn all(x: f32) -> Self {
        Self {
            bottom_left: x,
            bottom_right: x,
            top_right: x,
            top_left: x,
        }
    }

    pub fn to_vec4(&self) -> Vec4 {
        Vec4::new(self.bottom_left, self.bottom_right, self.top_right, self.top_left)
    }
}

impl Default for BorderRadius {
    fn default() -> Self {
        Self {
            bottom_left: 0.0,
            bottom_right: 0.0,
            top_right: 0.0,
            top_left: 0.0,
        }
    }
}
