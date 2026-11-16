//! Interactive application state and key handling. Drawing lives in `render`.

mod active_input;
mod popup;
mod render;

use jiff::civil::Date;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use active_input::ActiveInput;
use popup::{Popup, PopupAction};

use crate::domain::{Period, TimeOfDay, Timesheet, calendar};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Tracking,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Quit,
}

/// The panel of the Tracking screen that has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Days,
    Periods,
}

/// A focusable field of a period, in Tab order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    From,
    To,
    Project,
    Notes,
    Break,
    Vacation,
    Sick,
    Holiday,
}

impl Field {
    const ALL: [Field; 8] = [
        Field::From,
        Field::To,
        Field::Project,
        Field::Notes,
        Field::Break,
        Field::Vacation,
        Field::Sick,
        Field::Holiday,
    ];

    fn index(self) -> usize {
        Self::ALL.iter().position(|&f| f == self).expect("listed")
    }

    fn next(self) -> Option<Field> {
        Self::ALL.get(self.index() + 1).copied()
    }

    fn previous(self) -> Option<Field> {
        self.index().checked_sub(1).map(|i| Self::ALL[i])
    }

    /// From and To take times typed as HHMM.
    fn is_time(self) -> bool {
        matches!(self, Field::From | Field::To)
    }

    /// Fields that take typed text (as opposed to checkboxes).
    fn takes_text(self) -> bool {
        matches!(
            self,
            Field::From | Field::To | Field::Project | Field::Notes
        )
    }

    /// Whether typing `c` adds it to an active input of `len` characters:
    /// time fields take at most four digits, other text fields anything.
    fn accepts(self, c: char, len: usize) -> bool {
        self.takes_text() && (!self.is_time() || (c.is_ascii_digit() && len < 4))
    }
}

/// Which way focus moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Backward,
    Forward,
}

/// The focused field of the focused period in the right panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldFocus {
    pub period: usize,
    pub field: Field,
}

pub struct App {
    timesheet: Timesheet,
    today: Date,
    screen: Screen,
    panel: Panel,
    selected: Date,
    focus: Option<FieldFocus>,
    /// The focused field is being edited while this is set.
    active_input: Option<ActiveInput>,
    /// A one-line notice in the right panel; cleared at the next key press.
    message: Option<String>,
    popup: Option<Popup>,
    /// `None` until the first render.
    first_visible_day: Option<Date>,
    first_visible_period: usize,
}

impl App {
    pub fn new(timesheet: Timesheet, today: Date) -> Self {
        Self {
            timesheet,
            today,
            screen: Screen::Tracking,
            panel: Panel::Days,
            selected: calendar::start_day(today),
            focus: None,
            active_input: None,
            message: None,
            popup: None,
            first_visible_day: None,
            first_visible_period: 0,
        }
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn panel(&self) -> Panel {
        self.panel
    }

    pub fn selected_day(&self) -> Date {
        self.selected
    }

    pub fn focus(&self) -> Option<FieldFocus> {
        self.focus
    }

    pub fn is_editing(&self) -> bool {
        self.active_input.is_some()
    }

    pub fn timesheet(&self) -> &Timesheet {
        &self.timesheet
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        self.message = None;
        if let Some(popup) = &mut self.popup {
            if let Some(action) = popup.handle_key(key.code) {
                self.popup = None;
                self.run_popup_action(action);
            }
            return Outcome::Continue;
        }
        if self.active_input.is_some() {
            self.handle_editing_key(key);
            return Outcome::Continue;
        }
        if ctrl && key.code == KeyCode::Char('q') {
            return Outcome::Quit;
        }
        match (self.screen, key.code) {
            (_, KeyCode::F(1)) => self.screen = Screen::Help,
            (_, KeyCode::F(2)) | (Screen::Help, KeyCode::Esc) => self.screen = Screen::Tracking,
            (Screen::Help, _) => {}
            (Screen::Tracking, KeyCode::Left) if alt => self.panel = Panel::Days,
            (Screen::Tracking, KeyCode::Right) if alt => self.panel = Panel::Periods,
            (Screen::Tracking, code) => match self.panel {
                Panel::Days => self.handle_days_key(code),
                Panel::Periods => self.handle_periods_key(code, ctrl, alt),
            },
        }
        Outcome::Continue
    }

    fn handle_days_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Up => self.select_day(calendar::previous_weekday(self.selected)),
            KeyCode::Down => self.select_day(calendar::next_weekday(self.selected)),
            KeyCode::PageUp => self.select_day(calendar::previous_week(self.selected)),
            KeyCode::PageDown => self.select_day(calendar::next_week(self.selected)),
            _ => {}
        }
    }

    fn handle_periods_key(&mut self, code: KeyCode, ctrl: bool, alt: bool) {
        match code {
            KeyCode::Up if ctrl => self.select_day(calendar::previous_weekday(self.selected)),
            KeyCode::Down if ctrl => self.select_day(calendar::next_weekday(self.selected)),
            KeyCode::Up => self.move_focus_vertically(Direction::Backward),
            KeyCode::Down => self.move_focus_vertically(Direction::Forward),
            KeyCode::Tab => self.move_focus_horizontally(Direction::Forward),
            KeyCode::BackTab => self.move_focus_horizontally(Direction::Backward),
            KeyCode::Enter => self.add_period_on_enter(),
            KeyCode::Delete if ctrl && self.focus.is_some() => {
                self.popup = Some(Popup::delete_period());
            }
            KeyCode::Char(c) if !ctrl && !alt => self.start_editing(c),
            _ => {}
        }
    }

    /// While editing, only editing keys work; Enter/Tab accept, Esc rejects.
    fn handle_editing_key(&mut self, key: KeyEvent) {
        let plain = !key
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT);
        let Some(field) = self.focus.map(|f| f.field) else {
            return;
        };
        let Some(input) = &mut self.active_input else {
            return;
        };
        match key.code {
            KeyCode::Char(c) if plain && field.accepts(c, input.len()) => input.insert(c),
            KeyCode::Backspace => input.backspace(),
            KeyCode::Delete => input.delete(),
            KeyCode::Left => input.left(),
            KeyCode::Right => input.right(),
            KeyCode::Home => input.home(),
            KeyCode::End => input.end(),
            KeyCode::Esc => self.active_input = None,
            KeyCode::Enter => {
                self.accept();
            }
            KeyCode::Tab | KeyCode::BackTab => {
                let direction = if key.code == KeyCode::Tab {
                    Direction::Forward
                } else {
                    Direction::Backward
                };
                if self.accept() {
                    self.move_focus_horizontally(direction);
                }
            }
            _ => {}
        }
    }

    /// Typing in a focused text field replaces its value with active input.
    fn start_editing(&mut self, c: char) {
        let Some(focus) = self.focus else {
            return;
        };
        if focus.field.accepts(c, 0) {
            self.active_input = Some(ActiveInput::new(c));
        }
    }

    /// Stores the active input in the focused field. An invalid time keeps the
    /// active input and shows a message. Returns whether it was accepted.
    fn accept(&mut self) -> bool {
        let (Some(focus), Some(input)) = (self.focus, &self.active_input) else {
            return false;
        };
        let text = input.text();
        let time = if focus.field.is_time() && !text.is_empty() {
            match TimeOfDay::parse_hhmm(&text) {
                Some(time) => Some(time),
                None => {
                    self.message = Some(format!("{text} is not a valid time (HHMM, 0000–2359)"));
                    return false;
                }
            }
        } else {
            None
        };
        let value = (!text.is_empty()).then_some(text);
        if let Some(period) = self.focused_period_mut() {
            match focus.field {
                Field::From => period.from = time,
                Field::To => period.to = time,
                Field::Project => period.project = value,
                Field::Notes => period.notes = value,
                _ => {}
            }
        }
        self.active_input = None;
        true
    }

    fn focused_period_mut(&mut self) -> Option<&mut Period> {
        let focus = self.focus?;
        self.timesheet
            .day_mut(self.selected)
            .periods
            .get_mut(focus.period)
    }

    /// Enter on a day without periods, without focus, or in the last period adds
    /// a period and focuses its From, unless the last period is still empty.
    fn add_period_on_enter(&mut self) {
        let count = self.period_count();
        let in_last_period = self.focus.is_some_and(|f| f.period + 1 == count);
        if count > 0 && self.focus.is_some() && !in_last_period {
            return;
        }
        let last_is_empty = self
            .timesheet
            .day(self.selected)
            .periods
            .last()
            .is_some_and(Period::is_empty);
        if last_is_empty {
            return;
        }
        let period = self.timesheet.day_mut(self.selected).add_period();
        self.focus = Some(FieldFocus {
            period,
            field: Field::From,
        });
    }

    fn run_popup_action(&mut self, action: PopupAction) {
        match action {
            PopupAction::DeletePeriod => self.delete_focused_period(),
            PopupAction::Cancel => {}
        }
    }

    /// Deletes the focused period; focus moves to the period taking its place.
    fn delete_focused_period(&mut self) {
        let Some(focus) = self.focus else {
            return;
        };
        let day = self.timesheet.day_mut(self.selected);
        day.remove_period(focus.period);
        let count = day.periods.len();
        self.focus = (count > 0).then(|| FieldFocus {
            period: focus.period.min(count - 1),
            ..focus
        });
        self.first_visible_period = self.first_visible_period.min(count.saturating_sub(1));
    }

    /// Selects another day; no period or field has focus on the new day.
    fn select_day(&mut self, date: Date) {
        self.selected = date;
        self.focus = None;
        self.first_visible_period = 0;
    }

    fn period_count(&self) -> usize {
        self.timesheet.day(self.selected).periods.len()
    }

    /// ↑/↓: the same field in the previous/next period. Without focus, the first
    /// field of the last (↑) or first (↓) period.
    fn move_focus_vertically(&mut self, direction: Direction) {
        let down = direction == Direction::Forward;
        let count = self.period_count();
        if count == 0 {
            return;
        }
        self.focus = Some(match self.focus {
            None => FieldFocus {
                period: if down { 0 } else { count - 1 },
                field: Field::From,
            },
            Some(focus) => FieldFocus {
                period: if down {
                    (focus.period + 1).min(count - 1)
                } else {
                    focus.period.saturating_sub(1)
                },
                ..focus
            },
        });
    }

    /// Tab/Shift-Tab: the next/previous field, continuing in the next/previous
    /// period. Focus stays put at the very first and last field.
    fn move_focus_horizontally(&mut self, direction: Direction) {
        let forward = direction == Direction::Forward;
        let count = self.period_count();
        if count == 0 {
            return;
        }
        let first = FieldFocus {
            period: 0,
            field: Field::From,
        };
        let last = FieldFocus {
            period: count - 1,
            field: Field::Holiday,
        };
        self.focus = Some(match self.focus {
            None if forward => first,
            None => last,
            Some(focus) if forward => match focus.field.next() {
                Some(field) => FieldFocus { field, ..focus },
                None if focus.period + 1 < count => FieldFocus {
                    period: focus.period + 1,
                    field: Field::From,
                },
                None => focus,
            },
            Some(focus) => match focus.field.previous() {
                Some(field) => FieldFocus { field, ..focus },
                None if focus.period > 0 => FieldFocus {
                    period: focus.period - 1,
                    field: Field::Holiday,
                },
                None => focus,
            },
        });
    }
}
