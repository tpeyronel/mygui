use std::fmt::Debug;

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct FontWeight(u32);

impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200);
    pub const LIGHT: FontWeight = FontWeight(300);
    pub const REGULAR: FontWeight = FontWeight(400);
    pub const MEDIUM: FontWeight = FontWeight(500);
    pub const SEMI_BOLD: FontWeight = FontWeight(600);
    pub const BOLD: FontWeight = FontWeight(700);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800);
    pub const BLACK: FontWeight = FontWeight(900);

    #[allow(unused)]
    pub fn new(weight: u32) -> Self {
        Self(weight)
    }
}

impl Debug for FontWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            FontWeight::THIN => f.write_str("Thin"),
            FontWeight::EXTRA_LIGHT => f.write_str("ExtraLight"),
            FontWeight::LIGHT => f.write_str("Light"),
            FontWeight::REGULAR => f.write_str("Regular"),
            FontWeight::MEDIUM => f.write_str("Medium"),
            FontWeight::SEMI_BOLD => f.write_str("SemiBold"),
            FontWeight::BOLD => f.write_str("Bold"),
            FontWeight::EXTRA_BOLD => f.write_str("ExtraBold"),
            FontWeight::BLACK => f.write_str("Black"),
            _ => f.debug_tuple("FontWeight").field(&self.0).finish(),
        }
    }
}
