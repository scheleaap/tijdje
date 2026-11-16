//! Popups that ask the user to choose between labeled buttons.

use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAction {
    DeletePeriod,
    Save,
    DontSave,
    Overwrite,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Popup {
    pub question: &'static str,
    pub buttons: Vec<(&'static str, PopupAction)>,
    pub selected: usize,
}

impl Popup {
    pub fn delete_period() -> Self {
        Self {
            question: "Delete period?",
            buttons: vec![
                ("Delete", PopupAction::DeletePeriod),
                ("Cancel", PopupAction::Cancel),
            ],
            selected: 0,
        }
    }

    pub fn save_before_quitting() -> Self {
        Self {
            question: "Save before quitting?",
            buttons: vec![
                ("Save", PopupAction::Save),
                ("Don't save", PopupAction::DontSave),
                ("Cancel", PopupAction::Cancel),
            ],
            selected: 0,
        }
    }

    pub fn file_changed_on_disk() -> Self {
        Self {
            question: "File changed on disk",
            buttons: vec![
                ("Overwrite", PopupAction::Overwrite),
                ("Cancel", PopupAction::Cancel),
            ],
            selected: 0,
        }
    }

    /// ←/→ select a button, Enter or a button's first letter chooses it, and
    /// Esc cancels. Returns the chosen action, if any.
    pub fn handle_key(&mut self, code: KeyCode) -> Option<PopupAction> {
        match code {
            KeyCode::Left => self.selected = self.selected.saturating_sub(1),
            KeyCode::Right => self.selected = (self.selected + 1).min(self.buttons.len() - 1),
            KeyCode::Enter => return Some(self.buttons[self.selected].1),
            KeyCode::Esc => return Some(PopupAction::Cancel),
            KeyCode::Char(c) => {
                return self
                    .buttons
                    .iter()
                    .find(|(label, _)| {
                        label
                            .chars()
                            .next()
                            .is_some_and(|l| l.eq_ignore_ascii_case(&c))
                    })
                    .map(|&(_, action)| action);
            }
            _ => {}
        }
        None
    }
}
