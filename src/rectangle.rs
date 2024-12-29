use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
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

    pub fn bottom_right(&self) -> Vec2 {
        Vec2::new(self.right, self.bottom)
    }

    pub fn top_left(&self) -> Vec2 {
        Vec2::new(self.left, self.top)
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

    #[allow(unused)]
    pub fn size(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    pub fn contains(&self, point: Vec2) -> bool {
        (self.left <= point.x && point.x <= self.right) && (self.bottom <= point.y && point.y <= self.top)
    }
}
