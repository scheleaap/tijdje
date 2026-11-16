//! Active input: text typed into a field that hasn't been accepted or rejected yet.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveInput {
    text: Vec<char>,
    /// Cursor position, in characters.
    cursor: usize,
    /// First visible character when the text is wider than its field.
    scroll: usize,
}

impl ActiveInput {
    /// Starts editing with the first typed character, replacing the old value.
    pub fn new(first: char) -> Self {
        Self {
            text: vec![first],
            cursor: 1,
            scroll: 0,
        }
    }

    pub fn text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn len(&self) -> usize {
        self.text.len()
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
