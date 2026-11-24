//! Active input: text typed into a field that hasn't been accepted or rejected yet.

use crate::words;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveInput {
    text: Vec<char>,
    /// Cursor position, in characters.
    cursor: usize,
    /// First visible character when the text is wider than its field.
    scroll: usize,
}

impl ActiveInput {
    /// Starts editing with `text`, the cursor at the end.
    pub fn with_text(text: &str) -> Self {
        let text: Vec<char> = text.chars().collect();
        Self {
            cursor: text.len(),
            text,
            scroll: 0,
        }
    }

    pub fn text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.text.len()
    }

    /// Whether the cursor is after the last character.
    pub fn is_cursor_at_end(&self) -> bool {
        self.cursor == self.text.len()
    }

    pub fn insert(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += 1;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.text.remove(self.cursor);
        }
    }

    pub fn delete(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    pub fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.text.len());
    }

    pub fn home(&mut self) {
        self.cursor = 0;
    }

    pub fn end(&mut self) {
        self.cursor = self.text.len();
    }

    /// Moves the cursor to the start of the previous word.
    pub fn word_left(&mut self) {
        self.cursor = words::previous_word_start(&self.text, self.cursor);
    }

    /// Moves the cursor to the end of the next word.
    pub fn word_right(&mut self) {
        self.cursor = words::next_word_end(&self.text, self.cursor);
    }

    /// Deletes from the start of the previous word up to the cursor.
    pub fn delete_previous_word(&mut self) {
        let start = words::previous_word_start(&self.text, self.cursor);
        self.text.drain(start..self.cursor);
        self.cursor = start;
    }

    /// Deletes everything before the cursor.
    pub fn delete_to_start(&mut self) {
        self.text.drain(..self.cursor);
        self.cursor = 0;
    }

    /// Deletes everything from the cursor on.
    pub fn delete_to_end(&mut self) {
        self.text.truncate(self.cursor);
    }

    /// The part of the text that fits in a field `width` characters wide,
    /// scrolled so the cursor stays visible, and the cursor's column in it.
    /// At the end of the text, the cursor may sit just after the field.
    pub fn visible(&mut self, width: usize) -> (String, usize) {
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor > self.scroll + width {
            self.scroll = self.cursor - width;
        }
        let shown = self.text.iter().skip(self.scroll).take(width).collect();
        (shown, self.cursor - self.scroll)
    }
}
