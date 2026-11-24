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

/// A popup with labeled buttons, each standing for an action of type `A`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Popup<A = PopupAction> {
    /// A short title, shown in the frame.
    pub title: &'static str,
    /// The full text, shown inside and wrapped if needed.
    pub message: String,
    pub buttons: Vec<(&'static str, A)>,
    pub selected: usize,
}

impl<A: Copy> Popup<A> {
    /// The index of the selected button.
    pub fn selected_index(&self) -> usize {
        self.selected
    }

    /// How many buttons the popup has.
    pub fn button_count(&self) -> usize {
        self.buttons.len()
    }

    /// Selects the button at `index`, or the last one if there aren't that many.
    pub fn select(&mut self, index: usize) {
        self.selected = index.min(self.buttons.len() - 1);
    }

    /// Selects the button to the left, stopping at the first.
    pub fn select_previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Selects the button to the right, stopping at the last.
    pub fn select_next(&mut self) {
        self.select(self.selected + 1);
    }

    /// The selected button's action.
    pub fn selected_action(&self) -> A {
        self.buttons[self.selected].1
    }

    /// The action of the button whose label starts with `letter`, in any case.
    pub fn action_for_letter(&self, letter: char) -> Option<A> {
        self.buttons
            .iter()
            .find(|(label, _)| {
                label
                    .chars()
                    .next()
                    .is_some_and(|l| l.eq_ignore_ascii_case(&letter))
            })
            .map(|&(_, action)| action)
    }
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
            KeyCode::Left => {
                self.select_previous();
                None
            }
            KeyCode::Right => {
                self.select_next();
                None
            }
            KeyCode::Enter => Some(self.selected_action()),
            KeyCode::Esc => Some(PopupAction::Cancel),
            KeyCode::Char(c) => self.action_for_letter(c),
            _ => None,
        }
    }
}
