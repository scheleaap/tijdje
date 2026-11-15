//! Interactive application state and key handling. Drawing lives in `render`.

mod render;

use jiff::civil::Date;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::domain::{Timesheet, calendar};

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

    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
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
                Panel::Periods => self.handle_periods_key(code, ctrl),
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

    fn handle_periods_key(&mut self, code: KeyCode, ctrl: bool) {
        match code {
            KeyCode::Up if ctrl => self.select_day(calendar::previous_weekday(self.selected)),
            KeyCode::Down if ctrl => self.select_day(calendar::next_weekday(self.selected)),
            KeyCode::Up => self.move_focus_vertically(Direction::Backward),
            KeyCode::Down => self.move_focus_vertically(Direction::Forward),
            KeyCode::Tab => self.move_focus_horizontally(Direction::Forward),
            KeyCode::BackTab => self.move_focus_horizontally(Direction::Backward),
            _ => {}
        }
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
