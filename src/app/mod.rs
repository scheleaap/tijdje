//! Interactive application state and key handling. Drawing lives in `render`.

mod active_input;
mod popup;
mod render;

use std::{collections::HashSet, path::PathBuf};

use jiff::civil::Date;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use active_input::ActiveInput;
use popup::{Popup, PopupAction};

use crate::{
    domain::{
        ConsistencyProblem, Day, InconsistentDay, Period, PeriodType, TimeOfDay, Timesheet,
        calendar,
    },
    store::{self, Backup, FileSystem, Fingerprint, LoadError, Overwrite, SaveError},
};

/// Where the timesheet is loaded from and saved to.
struct Storage {
    fs: Box<dyn FileSystem>,
    path: PathBuf,
    /// The file as last loaded or saved, to detect changes made outside the app.
    fingerprint: Fingerprint,
    /// Whether this session already kept the original file as a backup.
    backup_made: bool,
}

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
    /// All fields in Tab order, which is their order on screen.
    const ALL: [Field; 8] = [
        Field::From,
        Field::To,
        Field::Project,
        Field::Break,
        Field::Vacation,
        Field::Sick,
        Field::Holiday,
        Field::Notes,
    ];
    const FIRST: Field = Field::ALL[0];
    const LAST: Field = Field::ALL[Field::ALL.len() - 1];

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

    /// The period-type checkboxes, in Tab order.
    pub const CHECKBOXES: [Field; 4] = [Field::Break, Field::Vacation, Field::Sick, Field::Holiday];

    /// The period type a checkbox field stands for.
    pub fn period_type(self) -> Option<PeriodType> {
        match self {
            Field::Break => Some(PeriodType::Break),
            Field::Vacation => Some(PeriodType::Vacation),
            Field::Sick => Some(PeriodType::Sick),
            Field::Holiday => Some(PeriodType::Holiday),
            Field::From | Field::To | Field::Project | Field::Notes => None,
        }
    }

    /// Fields that take typed text: every field that isn't a checkbox.
    fn takes_text(self) -> bool {
        self.period_type().is_none()
    }

    /// Whether typing `c` adds it to an active input of `len` characters:
    /// time fields take at most four digits, other text fields anything.
    fn accepts(self, c: char, len: usize) -> bool {
        self.takes_text() && (!self.is_time() || (c.is_ascii_digit() && len < 4))
    }
}

/// Alt shortcuts that act on the focused period (Alt+b/v/s/h/p/n).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeriodShortcut {
    Toggle(PeriodType),
    Focus(Field),
}

impl PeriodShortcut {
    fn for_key(code: KeyCode) -> Option<Self> {
        Some(match code {
            KeyCode::Char('b') => Self::Toggle(PeriodType::Break),
            KeyCode::Char('v') => Self::Toggle(PeriodType::Vacation),
            KeyCode::Char('s') => Self::Toggle(PeriodType::Sick),
            KeyCode::Char('h') => Self::Toggle(PeriodType::Holiday),
            KeyCode::Char('p') => Self::Focus(Field::Project),
            KeyCode::Char('n') => Self::Focus(Field::Notes),
            _ => return None,
        })
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
    storage: Storage,
    /// Whether the timesheet has changes that aren't saved yet.
    has_unsaved_changes: bool,
    /// Set when "Save" was chosen in the "Save before quitting?" popup.
    quit_after_saving: bool,
    today: Date,
    screen: Screen,
    panel: Panel,
    selected: Date,
    focus: Option<FieldFocus>,
    /// The focused field is being edited while this is set.
    active_input: Option<ActiveInput>,
    /// A one-line notice in the right panel; cleared at the next key press.
    message: Option<String>,
    /// Periods of the selected day that failed the consistency check; cleared
    /// at the next key press, like the message.
    inconsistent_periods: Vec<usize>,
    /// Days changed in this session; only these are checked when leaving them.
    changed_days: HashSet<Date>,
    /// The day named in a consistency-check popup, for "Go to day".
    inconsistent_day: Option<InconsistentDay>,
    popup: Option<Popup>,
    /// `None` until the first render.
    first_visible_day: Option<Date>,
    first_visible_period_row: usize,
}

impl App {
    /// Loads the timesheet at `path` and starts on `today`.
    pub fn open(fs: Box<dyn FileSystem>, path: PathBuf, today: Date) -> Result<Self, LoadError> {
        let (timesheet, fingerprint) = store::load(fs.as_ref(), &path)?;
        Ok(Self::new(
            timesheet,
            Storage {
                fs,
                path,
                fingerprint,
                backup_made: false,
            },
            today,
        ))
    }

    fn new(timesheet: Timesheet, storage: Storage, today: Date) -> Self {
        Self {
            timesheet,
            storage,
            has_unsaved_changes: false,
            quit_after_saving: false,
            today,
            screen: Screen::Tracking,
            panel: Panel::Days,
            selected: calendar::start_day(today),
            focus: None,
            active_input: None,
            message: None,
            inconsistent_periods: Vec::new(),
            changed_days: HashSet::new(),
            inconsistent_day: None,
            popup: None,
            first_visible_day: None,
            first_visible_period_row: 0,
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

    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        self.message = None;
        self.inconsistent_periods.clear();
        if let Some(popup) = &mut self.popup {
            if let Some(action) = popup.handle_key(key.code) {
                self.popup = None;
                return self.run_popup_action(action);
            }
            return Outcome::Continue;
        }
        if alt
            && self.screen == Screen::Tracking
            && self.panel == Panel::Periods
            && let Some(shortcut) = PeriodShortcut::for_key(key.code)
        {
            // Accept active input first; if that fails, the shortcut does nothing.
            if self.active_input.is_none() || self.accept() {
                self.run_period_shortcut(shortcut);
            }
            return Outcome::Continue;
        }
        if self.active_input.is_some() {
            self.handle_editing_key(key);
            return Outcome::Continue;
        }
        if ctrl && key.code == KeyCode::Char('q') {
            if self.check_all_days(("Quit anyway", PopupAction::QuitAnyway)) {
                return self.quit();
            }
            return Outcome::Continue;
        }
        if ctrl && key.code == KeyCode::Char('s') {
            if self.check_all_days(("Save anyway", PopupAction::SaveAnyway)) {
                return self.save(Overwrite::IfUnchanged);
            }
            return Outcome::Continue;
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
            KeyCode::Up => self.leave_day_for(calendar::previous_weekday(self.selected)),
            KeyCode::Down => self.leave_day_for(calendar::next_weekday(self.selected)),
            KeyCode::PageUp => self.leave_day_for(calendar::previous_week(self.selected)),
            KeyCode::PageDown => self.leave_day_for(calendar::next_week(self.selected)),
            _ => {}
        }
    }

    fn handle_periods_key(&mut self, code: KeyCode, ctrl: bool, alt: bool) {
        match code {
            KeyCode::Up if ctrl => self.leave_day_for(calendar::previous_weekday(self.selected)),
            KeyCode::Down if ctrl => self.leave_day_for(calendar::next_weekday(self.selected)),
            KeyCode::Up => self.move_focus_vertically(Direction::Backward),
            KeyCode::Down => self.move_focus_vertically(Direction::Forward),
            KeyCode::Tab => self.move_focus_horizontally(Direction::Forward),
            KeyCode::BackTab => self.move_focus_horizontally(Direction::Backward),
            KeyCode::Enter => self.add_period_on_enter(),
            KeyCode::Delete if ctrl && self.focus.is_some() => {
                let label = self.focused_period().map(Period::label).unwrap_or_default();
                self.popup = Some(Popup::delete_period(&label));
            }
            KeyCode::Char(c) if !ctrl && !alt => self.type_char(c),
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

    fn focused_period(&self) -> Option<&Period> {
        let focus = self.focus?;
        self.timesheet.day(self.selected).periods.get(focus.period)
    }

    /// Space toggles a focused checkbox; otherwise typing starts editing.
    fn type_char(&mut self, c: char) {
        match self.focus.and_then(|focus| focus.field.period_type()) {
            Some(period_type) if c == ' ' => self.toggle_period_type(period_type),
            _ => self.start_editing(c),
        }
    }

    fn toggle_period_type(&mut self, period_type: PeriodType) {
        if let Some(focus) = self.focus {
            self.edit_selected_day(|day| day.toggle_period_type(focus.period, period_type));
        }
    }

    fn run_period_shortcut(&mut self, shortcut: PeriodShortcut) {
        let Some(focus) = self.focus else {
            return;
        };
        match shortcut {
            PeriodShortcut::Toggle(period_type) => self.toggle_period_type(period_type),
            PeriodShortcut::Focus(field) => self.focus = Some(FieldFocus { field, ..focus }),
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
        self.edit_selected_day(|day| {
            if let Some(period) = day.periods.get_mut(focus.period) {
                match focus.field {
                    Field::From => period.from = time,
                    Field::To => period.to = time,
                    Field::Project => period.project = value,
                    Field::Notes => period.notes = value,
                    _ => {}
                }
            }
        });
        self.active_input = None;
        true
    }

    /// Changes the selected day. This is the only way the timesheet changes, so
    /// unsaved changes are tracked here, and only when something did change.
    fn edit_selected_day<R>(&mut self, edit: impl FnOnce(&mut Day) -> R) -> R {
        let day = self.timesheet.day_mut(self.selected);
        let before = day.clone();
        let result = edit(day);
        if *day != before {
            self.has_unsaved_changes = true;
            self.changed_days.insert(self.selected);
        }
        result
    }

    /// Selects `date`, unless the selected day was changed in this session
    /// and fails the consistency check: then it stays selected, with a
    /// message and its offending periods highlighted. Either way, the
    /// selected day is sorted and its empty periods removed first.
    fn leave_day_for(&mut self, date: Date) {
        if self.changed_days.contains(&self.selected) {
            self.normalize_selected_day();
            if let Some(problem) = self
                .timesheet
                .day(self.selected)
                .check(self.selected, self.today)
            {
                self.show_problem(problem);
                return;
            }
        }
        self.select_day(date);
    }

    fn normalize_selected_day(&mut self) {
        let moved = self.edit_selected_day(Day::normalize);
        self.follow_moved_periods(&moved);
    }

    /// Keeps focus on the same period after the selected day was sorted.
    fn follow_moved_periods(&mut self, moved: &[Option<usize>]) {
        self.focus = self.focus.and_then(|focus| {
            moved
                .get(focus.period)
                .copied()
                .flatten()
                .map(|period| FieldFocus { period, ..focus })
        });
    }

    fn show_problem(&mut self, problem: ConsistencyProblem) {
        self.message = Some(problem.message);
        self.inconsistent_periods = problem.periods;
    }

    /// Sorts every day and removes empty periods, then checks all days. For
    /// the earliest failing day, shows a popup with `anyway` as the button
    /// that continues regardless; returns whether all days passed. Sorting
    /// alone isn't an unsaved change.
    fn check_all_days(&mut self, anyway: (&'static str, PopupAction)) -> bool {
        let mut moved_on_selected_day = Vec::new();
        for (&date, day) in &mut self.timesheet.days {
            let moved = day.normalize();
            if date == self.selected {
                moved_on_selected_day = moved;
            }
        }
        self.follow_moved_periods(&moved_on_selected_day);
        match self.timesheet.first_inconsistent_day(self.today) {
            None => true,
            Some(inconsistent) => {
                let question = format!(
                    "{}: {}",
                    calendar::format_dutch_short(inconsistent.date),
                    inconsistent.problem.message
                );
                self.popup = Some(Popup::inconsistent_day(question, anyway));
                self.inconsistent_day = Some(inconsistent);
                false
            }
        }
    }

    /// Quits, asking first whether to save unsaved changes.
    fn quit(&mut self) -> Outcome {
        if !self.has_unsaved_changes {
            return Outcome::Quit;
        }
        self.popup = Some(Popup::save_before_quitting());
        Outcome::Continue
    }

    /// Selects the day from a consistency-check popup and shows its problem.
    fn go_to_inconsistent_day(&mut self) {
        if let Some(inconsistent) = self.inconsistent_day.take() {
            self.select_day(inconsistent.date);
            self.panel = Panel::Periods;
            self.show_problem(inconsistent.problem);
        }
    }

    /// Enter on a day without periods, without focus, or in the last period adds
    /// a period and focuses its From, unless the last period is still open
    /// (which includes an empty period).
    fn add_period_on_enter(&mut self) {
        let count = self.period_count();
        let in_last_period = self.focus.is_some_and(|f| f.period + 1 == count);
        if count > 0 && self.focus.is_some() && !in_last_period {
            return;
        }
        let last_is_open = self
            .timesheet
            .day(self.selected)
            .periods
            .last()
            .is_some_and(Period::is_open);
        if last_is_open {
            return;
        }
        let period = self.edit_selected_day(Day::add_period);
        self.focus = Some(FieldFocus {
            period,
            field: Field::From,
        });
    }

    fn run_popup_action(&mut self, action: PopupAction) -> Outcome {
        match action {
            PopupAction::DeletePeriod => self.delete_focused_period(),
            PopupAction::GoToDay => self.go_to_inconsistent_day(),
            PopupAction::SaveAnyway => return self.save(Overwrite::IfUnchanged),
            PopupAction::QuitAnyway => return self.quit(),
            PopupAction::Save => {
                self.quit_after_saving = true;
                return self.save(Overwrite::IfUnchanged);
            }
            PopupAction::DontSave => return Outcome::Quit,
            PopupAction::Overwrite => return self.save(Overwrite::Always),
            PopupAction::Cancel => self.quit_after_saving = false,
        }
        Outcome::Continue
    }

    /// Saves the timesheet. If the file changed on disk, asks whether to
    /// overwrite it, unless `overwrite` is `Always`. The first save of a
    /// session keeps the file as it was when the application started as a
    /// backup.
    fn save(&mut self, overwrite: Overwrite) -> Outcome {
        let storage = &mut self.storage;
        let backup = if storage.backup_made {
            Backup::Skip
        } else {
            Backup::Create
        };
        match store::save(
            storage.fs.as_ref(),
            &storage.path,
            &self.timesheet,
            &storage.fingerprint,
            overwrite,
            backup,
        ) {
            Ok(fingerprint) => {
                storage.fingerprint = fingerprint;
                storage.backup_made = true;
                self.has_unsaved_changes = false;
                if self.quit_after_saving {
                    return Outcome::Quit;
                }
            }
            Err(SaveError::ChangedOnDisk) => self.popup = Some(Popup::file_changed_on_disk()),
            Err(e @ SaveError::Io(_)) => {
                self.message = Some(e.to_string());
                self.quit_after_saving = false;
            }
        }
        Outcome::Continue
    }

    /// Deletes the focused period; focus moves to the period taking its place.
    fn delete_focused_period(&mut self) {
        let Some(focus) = self.focus else {
            return;
        };
        let count = self.edit_selected_day(|day| {
            day.remove_period(focus.period);
            day.periods.len()
        });
        self.focus = (count > 0).then(|| FieldFocus {
            period: focus.period.min(count - 1),
            ..focus
        });
    }

    /// Selects another day; no period or field has focus on the new day.
    fn select_day(&mut self, date: Date) {
        self.selected = date;
        self.focus = None;
        self.first_visible_period_row = 0;
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
            field: Field::FIRST,
        };
        let last = FieldFocus {
            period: count - 1,
            field: Field::LAST,
        };
        self.focus = Some(match self.focus {
            None if forward => first,
            None => last,
            Some(focus) if forward => match focus.field.next() {
                Some(field) => FieldFocus { field, ..focus },
                None if focus.period + 1 < count => FieldFocus {
                    period: focus.period + 1,
                    field: Field::FIRST,
                },
                None => focus,
            },
            Some(focus) => match focus.field.previous() {
                Some(field) => FieldFocus { field, ..focus },
                None if focus.period > 0 => FieldFocus {
                    period: focus.period - 1,
                    field: Field::LAST,
                },
                None => focus,
            },
        });
    }
}
