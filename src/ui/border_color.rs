use crate::color::Color;

#[derive(Debug, Clone, PartialEq)]
pub struct BorderColor {
    left: Color,
    bottom: Color,
    right: Color,
    top: Color,
}

impl BorderColor {
    pub fn new(left: Color, bottom: Color, right: Color, top: Color) -> Self {
        Self {
            left,
            bottom,
            right,
            top,
        }
    }

    pub fn all(color: Color) -> Self {
        Self::new(color, color, color, color)
    }

    pub fn left(&self) -> &Color {
        &self.left
    }

    pub fn bottom(&self) -> &Color {
        &self.bottom
    }

    pub fn right(&self) -> &Color {
        &self.right
    }

    pub fn top(&self) -> &Color {
        &self.top
    }
}

impl Default for BorderColor {
    fn default() -> Self {
        Self::all(Color::default())
    }
}

impl From<Color> for BorderColor {
    fn from(value: Color) -> Self {
        Self::all(value)
    }
}
