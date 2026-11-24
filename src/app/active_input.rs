//! Active input: text typed into a field that hasn't been accepted or rejected yet.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::Modifiers;
use crate::words;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveInput {
    text: Vec<char>,
    /// Cursor position, in characters.
    cursor: usize,
    /// First visible character when the text is wider than its field.
    scroll: usize,
    /// Whether characters were typed or deleted since editing started.
    changed: bool,
}

impl ActiveInput {
    /// Starts editing with `text`, the cursor at the end.
    pub fn with_text(text: &str) -> Self {
        let text: Vec<char> = text.chars().collect();
        Self {
            cursor: text.len(),
            text,
            scroll: 0,
            changed: false,
        }
    }

    /// Replaces the text with `text`, the cursor at the end, as a change.
    pub fn replace_with(&mut self, text: &str) {
        *self = Self {
            changed: true,
            ..Self::with_text(text)
        };
    }

    /// Whether characters were typed or deleted since editing started.
    pub fn is_changed(&self) -> bool {
        self.changed
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
        self.changed = true;
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

    /// Applies a shell-like editing key that moves the cursor or deletes
    /// text; typed characters are left to the caller, which knows what the
    /// field takes. Returns whether `key` was such an editing key.
    pub fn apply_editing_key(&mut self, key: KeyEvent) -> bool {
        let edit: Option<fn(&mut Self)> = match (key.code, Modifiers::of(key)) {
            (KeyCode::Char('a'), Modifiers::Ctrl) => Some(Self::home),
            (KeyCode::Char('e'), Modifiers::Ctrl) => Some(Self::end),
            (KeyCode::Char('w'), Modifiers::Ctrl) => Some(Self::delete_previous_word),
            (KeyCode::Char('u'), Modifiers::Ctrl) => Some(Self::delete_to_start),
            (KeyCode::Char('k'), Modifiers::Ctrl) => Some(Self::delete_to_end),
            (KeyCode::Backspace, Modifiers::Ctrl) => Some(Self::delete_previous_word),
            (KeyCode::Backspace, Modifiers::None) => Some(Self::backspace),
            (KeyCode::Delete, Modifiers::None) => Some(Self::delete),
            (KeyCode::Left, Modifiers::Ctrl) => Some(Self::word_left),
            (KeyCode::Right, Modifiers::Ctrl) => Some(Self::word_right),
            (KeyCode::Left, Modifiers::None) => Some(Self::left),
            (KeyCode::Right, Modifiers::None) => Some(Self::right),
            (KeyCode::Home, Modifiers::None) => Some(Self::home),
            (KeyCode::End, Modifiers::None) => Some(Self::end),
            _ => None,
        };
        let length = self.text.len();
        let applied = edit.map(|edit| edit(self)).is_some();
        // Editing keys only delete, so a change shortens the text.
        self.changed |= self.text.len() != length;
        applied
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
