//! The Help window: every shortcut, grouped, shown over the Tracking screen.

use ratatui::crossterm::event::KeyCode;

/// A heading and its shortcuts, each a key and what it does.
type Group = (&'static str, &'static [(&'static str, &'static str)]);

/// The shortcuts, grouped as in the spec's Keyboard section.
const GROUPS: [Group; 6] = [
    (
        "General",
        &[
            ("F1", "Open/close this Help window"),
            ("F2", "Tracking screen"),
            ("Ctrl+S", "Save"),
            ("Ctrl+Q", "Quit"),
            ("Ctrl+Z/Ctrl+Y", "Undo/redo"),
            ("Esc", "Reject the input, close a dialog or this window"),
        ],
    ),
    (
        "Day navigation",
        &[
            ("Ctrl+↑/↓", "Previous/next day"),
            ("Ctrl+PgUp/PgDn", "Previous/next week"),
            ("Ctrl+G", "Go to date"),
        ],
    ),
    (
        "Field navigation",
        &[
            ("←/→/↑/↓", "Field left/right/above/below"),
            ("Tab/Shift-Tab", "Next/previous field"),
            ("PgUp/PgDn", "10 periods up/down"),
            ("Home/End", "First/last field of the period"),
            (
                "Ctrl+Enter",
                "From of the next period; in the last one: add a period",
            ),
        ],
    ),
    (
        "Field editing",
        &[
            (
                "Enter",
                "Edit the field or toggle the checkbox; while editing: accept",
            ),
            ("Type", "Replace the field's value"),
            (
                "Del",
                "Clear the field, uncheck the checkbox, or switch a part of day to from–to",
            ),
            ("Ctrl+C/Ctrl+V", "Copy/paste the field"),
            ("←/→", "While editing: move the cursor"),
            (
                "Home/End, Ctrl+A/E",
                "While editing: start/end of the input",
            ),
            ("Ctrl+←/→", "While editing: previous/next word"),
            ("Backspace/Del", "While editing: delete a character"),
            (
                "Ctrl+Backspace/Ctrl+W",
                "While editing: delete the previous word",
            ),
            ("Ctrl+U/Ctrl+K", "While editing: delete to the start/end"),
            (
                "Tab/Shift-Tab, Ctrl+Enter",
                "While editing: accept and move on",
            ),
            (
                "→",
                "Editing a project: fill in the project suggestion, keep editing",
            ),
            (
                "Tab/Shift-Tab, Ctrl+Enter",
                "Editing a project: take the project suggestion, accept and move on",
            ),
        ],
    ),
    (
        "Period definition",
        &[
            ("Alt+b/f/s/v/h", "Toggle break/free/sick/vacation/holiday"),
            (
                "Alt+0/1/2/3",
                "Switch to from–to/full day/1st half/2nd half",
            ),
        ],
    ),
    ("Period management", &[("Ctrl+Del", "Delete the period")]),
];

/// A line of the Help window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpLine {
    /// The heading of a group, e.g. "General".
    Heading(&'static str),
    /// A shortcut: its key and what it does.
    Shortcut {
        key: &'static str,
        action: &'static str,
    },
    /// The empty line between two groups.
    Blank,
}

/// The Help window's lines: each group's heading and shortcuts, with a
/// blank line between groups.
pub fn help_lines() -> Vec<HelpLine> {
    GROUPS
        .iter()
        .enumerate()
        .flat_map(|(i, (heading, shortcuts))| {
            (i > 0)
                .then_some(HelpLine::Blank)
                .into_iter()
                .chain([HelpLine::Heading(heading)])
                .chain(
                    shortcuts
                        .iter()
                        .map(|&(key, action)| HelpLine::Shortcut { key, action }),
                )
        })
        .collect()
}

/// The open Help window and how far it is scrolled.
#[derive(Debug, Default)]
pub struct HelpWindow {
    /// The index of the first line shown.
    first_line: usize,
    /// How many lines the window showed when it was last drawn.
    page_height: usize,
}

impl HelpWindow {
    /// Scrolls with ↑/↓ by a line, PgUp/PgDn by a page and Home/End to the
    /// top/bottom; other keys do nothing. Scrolling past the bottom is
    /// undone when the window is drawn.
    pub fn scroll(&mut self, code: KeyCode) {
        self.first_line = match code {
            KeyCode::Up => self.first_line.saturating_sub(1),
            KeyCode::Down => self.first_line.saturating_add(1),
            KeyCode::PageUp => self.first_line.saturating_sub(self.page_height),
            KeyCode::PageDown => self.first_line.saturating_add(self.page_height),
            KeyCode::Home => 0,
            KeyCode::End => usize::MAX,
            _ => self.first_line,
        };
    }

    /// The index of the first line shown.
    pub fn first_line(&self) -> usize {
        self.first_line
    }

    /// Fits the window to showing `line_count` lines in `height` rows: a
    /// page becomes `height` lines, and the last line never scrolls above
    /// the bottom row.
    pub fn fit(&mut self, line_count: usize, height: usize) {
        self.page_height = height;
        self.first_line = self.first_line.min(bottom_first_line(line_count, height));
    }
}

/// The index of the first line shown when `line_count` lines are scrolled
/// to the bottom of `height` rows.
fn bottom_first_line(line_count: usize, height: usize) -> usize {
    line_count.saturating_sub(height)
}
