use dyn_partial_eq::DynPartialEq;

use super::text_cursor::TextCursor;

#[derive(Debug, Clone, Copy, PartialEq, DynPartialEq, Eq, Hash)]
pub struct TextPosition {
    pub line: u32,
    pub column: u32,
}

impl TextPosition {
    pub fn from_text_index(text: &str, index: usize) -> TextPosition {
        let mut cursor = TextCursor::new(text);
        cursor.go_to_index(index);
        cursor.position()
    }
}
