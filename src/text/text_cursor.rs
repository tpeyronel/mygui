use super::{text_position::TextPosition, TAB_SIZE};

pub struct TextCursor<'a> {
    text: &'a str,
    index: usize,
    position: TextPosition,
}

impl<'a> TextCursor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            index: 0,
            position: TextPosition { line: 0, column: 0 },
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn position(&self) -> TextPosition {
        self.position
    }

    pub fn advance(&mut self) -> bool {
        let Some(c) = self.text[self.index as usize..].chars().next() else {
            return false;
        };

        match c {
            '\n' => {
                self.position.line += 1;
                self.position.column = 0;
            }
            '\t' => {
                self.position.column += TAB_SIZE - (self.position.column % TAB_SIZE);
            }
            _ => {
                self.position.column += 1;
            }
        }

        self.index += c.len_utf8();

        true
    }

    pub fn try_go_to_prev_new_line(&mut self) -> bool {
        let mut index = self.index;

        for c in self.text[..self.index as usize].chars().rev() {
            index -= c.len_utf8();

            if c == '\n' {
                self.index = index;
                self.position.line -= 1;
                self.position.column = 0;
                return true;
            }
        }

        false
    }

    pub fn go_to_line_start(&mut self) {
        for c in self.text[..self.index as usize].chars().rev() {
            if c == '\n' {
                break;
            }

            self.index -= c.len_utf8();
        }

        self.position.column = 0;
    }

    #[allow(unused)]
    pub fn go_down(&mut self) -> bool {
        let target_position = TextPosition {
            line: self.position.line + 1,
            column: self.position.column,
        };

        for c in self.text[self.index as usize..].chars() {
            // This should never panic due to special handling of newlines below.
            assert!(self.position.line <= target_position.line);

            let reached_line = self.position.line == target_position.line;
            let reached_column = self.position.column >= target_position.column;
            if reached_line && (reached_column || c == '\n') {
                break;
            }

            self.advance();
        }

        true
    }

    #[allow(unused)]
    pub fn go_up(&mut self) {
        let target_position = TextPosition {
            line: self.position.line - 1,
            column: self.position.column,
        };

        if !self.try_go_to_prev_new_line() {
            return;
        }

        assert_eq!(self.position.line, target_position.line);

        self.go_to_line_start();

        assert_eq!(self.position.line, target_position.line);
        assert_eq!(self.position.column, 0);

        self.go_forwards_to(target_position);
    }

    #[allow(unused)]
    fn go_forwards_to(&mut self, target_position: TextPosition) {
        if target_position.line == self.position.line {
            assert!(target_position.column >= self.position.column);
        } else {
            assert!(target_position.line > self.position.line);
        }

        for c in self.text[self.index as usize..].chars() {
            // This should never panic due to special handling of newlines below.
            assert!(self.position.line <= target_position.line);

            let reached_line = self.position.line == target_position.line;
            let reached_column = self.position.column >= target_position.column;
            if reached_line && (reached_column || c == '\n') {
                break;
            }

            self.advance();
        }
    }

    pub fn go_to_index(&mut self, target_index: usize) {
        // For now only support going forwards.
        assert!(target_index >= self.index);

        while self.index < target_index {
            self.advance();
        }

        assert_eq!(
            self.index, target_index,
            "invalid target index {} for string {}",
            target_index, self.text
        );
    }
}
