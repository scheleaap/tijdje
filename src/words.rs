//! Word boundaries for word-wise editing. A word is anything between spaces.

/// Where the word before `cursor` starts: spaces right before the cursor are
/// skipped first, then the word itself.
pub fn previous_word_start(text: &[char], cursor: usize) -> usize {
    let is_space = |i: usize| text[i - 1] == ' ';
    let mut i = cursor.min(text.len());
    while i > 0 && is_space(i) {
        i -= 1;
    }
    while i > 0 && !is_space(i) {
        i -= 1;
    }
    i
}

/// Where the word after `cursor` ends: spaces right after the cursor are
/// skipped first, then the word itself.
pub fn next_word_end(text: &[char], cursor: usize) -> usize {
    let is_space = |i: usize| text[i] == ' ';
    let mut i = cursor.min(text.len());
    while i < text.len() && is_space(i) {
        i += 1;
    }
    while i < text.len() && !is_space(i) {
        i += 1;
    }
    i
}
