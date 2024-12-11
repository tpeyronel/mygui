use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

impl Rectangle {
    pub fn from_position_size(position: Vec2, size: Vec2) -> Self {
        Self {
            left: position.x,
            bottom: position.y,
            right: position.x + size.x,
            top: position.y + size.y,
        }
    }

    pub fn bottom_left(&self) -> Vec2 {
        Vec2::new(self.left, self.bottom)
    }

    pub fn top_right(&self) -> Vec2 {
        Vec2::new(self.right, self.top)
    }

    pub fn width(&self) -> f32 {
        self.right - self.left
    }

    pub fn height(&self) -> f32 {
        self.top - self.bottom
    }

    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }
}
