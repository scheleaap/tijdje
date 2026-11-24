//! The "Go to date" dialog: an input field for a date and the buttons Go,
//! Today and Cancel.

use ratatui::crossterm::event::{KeyCode, KeyEvent};

use super::{Modifiers, active_input::ActiveInput, dialog::Dialog};

/// The date formats the input field takes, as shown to the user.
pub const DATE_FORMATS: &str = "YYYY-MM-DD, DD MMM or DD MMM YYYY";

/// A button of the dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoToDateButton {
    Go,
    Today,
    Cancel,
}

/// What the user chose in the dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GoToDateAction {
    /// Go to the date typed as this text.
    Go(String),
    /// Go to today.
    Today,
    /// Close the dialog without going anywhere.
    Cancel,
}

/// The dialog's state: the typed date, and whether the input field or one of
/// the buttons has focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoToDate {
    /// The title, the label above the input field and the buttons; its
    /// selected button is the focused one while the input field hasn't focus.
    pub dialog: Dialog<GoToDateButton>,
    /// The typed date.
    pub input: ActiveInput,
    /// Whether the input field has focus rather than a button.
    pub input_focused: bool,
}

impl GoToDate {
    /// The dialog as it opens: an empty input field with focus.
    pub fn new() -> Self {
        Self {
            dialog: Dialog {
                title: "Go to date",
                message: format!("Date ({DATE_FORMATS}):"),
                buttons: vec![
                    ("Go", GoToDateButton::Go),
                    ("Today", GoToDateButton::Today),
                    ("Cancel", GoToDateButton::Cancel),
                ],
                selected: 0,
            },
            input: ActiveInput::with_text(""),
            input_focused: true,
        }
    }

    /// Handles a key as in Midnight Commander's dialogs: Tab/Shift-Tab move
    /// focus between the input field and the buttons; typing goes to the
    /// focused input field; Alt and a button's first letter choose that
    /// button, and so does the plain letter while a button has focus; ←/→
    /// move between buttons; Enter presses the focused button, or Go in the
    /// input field; Esc cancels. Returns the choice, if any.
    pub fn handle_key(&mut self, key: KeyEvent) -> Option<GoToDateAction> {
        let modifiers = Modifiers::of(key);
        let button = match (key.code, modifiers) {
            (KeyCode::Esc, _) => Some(GoToDateButton::Cancel),
            (KeyCode::Tab, Modifiers::None) => self.move_focus(1),
            (KeyCode::BackTab, Modifiers::None) => self.move_focus(self.dialog.button_count()),
            (KeyCode::Char(c), Modifiers::Alt) => self.dialog.action_for_letter(c),
            _ if self.input_focused => self.handle_input_key(key, modifiers),
            (code, Modifiers::None) => self.handle_button_key(code),
            _ => None,
        };
        button.map(|button| self.action(button))
    }

    /// Keys while the input field has focus: Enter means Go, other keys edit
    /// the date. Ctrl with a letter types nothing.
    fn handle_input_key(&mut self, key: KeyEvent, modifiers: Modifiers) -> Option<GoToDateButton> {
        match (key.code, modifiers) {
            (KeyCode::Enter, Modifiers::None) => Some(GoToDateButton::Go),
            (KeyCode::Char(c), Modifiers::None) => {
                self.input.insert(c);
                None
            }
            _ => {
                self.input.apply_editing_key(key);
                None
            }
        }
    }

    /// Keys without modifiers while a button has focus.
    fn handle_button_key(&mut self, code: KeyCode) -> Option<GoToDateButton> {
        match code {
            KeyCode::Left => {
                self.dialog.select_previous();
                None
            }
            KeyCode::Right => {
                self.dialog.select_next();
                None
            }
            KeyCode::Enter => Some(self.dialog.selected_action()),
            KeyCode::Char(c) => self.dialog.action_for_letter(c),
            _ => None,
        }
    }

    /// Moves focus `steps` places forward through the input field and the
    /// buttons, wrapping around. Never chooses a button.
    fn move_focus(&mut self, steps: usize) -> Option<GoToDateButton> {
        let places = self.dialog.button_count() + 1;
        let current = if self.input_focused {
            0
        } else {
            self.dialog.selected_index() + 1
        };
        let next = (current + steps) % places;
        self.input_focused = next == 0;
        self.dialog.select(next.saturating_sub(1));
        None
    }

    /// What pressing `button` means; Go carries the typed text.
    fn action(&self, button: GoToDateButton) -> GoToDateAction {
        match button {
            GoToDateButton::Go => GoToDateAction::Go(self.input.text()),
            GoToDateButton::Today => GoToDateAction::Today,
            GoToDateButton::Cancel => GoToDateAction::Cancel,
        }
    }

    /// The input field while it has focus, for copying and pasting.
    pub fn focused_input(&mut self) -> Option<&mut ActiveInput> {
        self.input_focused.then_some(&mut self.input)
    }

    /// The selected button as shown: none while the input field has focus.
    pub fn shown_selected_button(&self) -> Option<usize> {
        (!self.input_focused).then_some(self.dialog.selected_index())
    }
}
