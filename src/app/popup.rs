//! Popups that ask the user to choose between labeled buttons.

use ratatui::crossterm::event::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupAction {
    DeletePeriod,
    GoToDay,
    SaveAnyway,
    QuitAnyway,
    Save,
    DontSave,
    Overwrite,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Popup {
    /// A short title, shown in the frame.
    pub title: &'static str,
    /// The full text, shown inside and wrapped if needed.
    pub message: String,
    pub buttons: Vec<(&'static str, PopupAction)>,
    pub selected: usize,
}

impl Popup {
    /// Asks to delete the period with times `label`, e.g. `0845–1230`.
    pub fn delete_period(label: &str) -> Self {
        Self {
            title: "Delete period",
            message: format!("Delete period {label}?"),
            buttons: vec![
                ("Delete", PopupAction::DeletePeriod),
                ("Cancel", PopupAction::Cancel),
            ],
            selected: 0,
        }
    }

    pub fn save_before_quitting() -> Self {
        Self {
            title: "Quit",
            message: "Save before quitting?".to_owned(),
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
            title: "File changed",
            message: "The file changed on disk since it was loaded or saved.".to_owned(),
            buttons: vec![
                ("Overwrite", PopupAction::Overwrite),
                ("Cancel", PopupAction::Cancel),
            ],
            selected: 0,
        }
    }

    /// A day fails the consistency check; `anyway` is the button that
    /// continues regardless ("Save anyway" or "Quit anyway").
    pub fn inconsistent_day(problem: String, anyway: (&'static str, PopupAction)) -> Self {
        Self {
            title: "Consistency",
            message: problem,
            buttons: vec![
                ("Go to day", PopupAction::GoToDay),
                anyway,
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
