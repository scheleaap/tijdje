//! Interactive application state and key handling. Drawing lives in `render`.

mod active_input;
mod history;
mod popup;
mod render;

use std::{borrow::Cow, collections::HashSet, path::PathBuf};

use jiff::civil::Date;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use active_input::ActiveInput;
use history::{Change, History};
use popup::{Popup, PopupAction};

use crate::{
    clipboard::{Clipboard, NoClipboard},
    clock::Clock,
    domain::{
        ConsistencyProblem, Day, InconsistentDay, Period, PeriodType, TimeOfDay, Timesheet,
        calendar,
    },
    store::{self, Backup, FileSystem, Fingerprint, LoadError, Overwrite, SaveError},
};

const NO_CLIPBOARD: &str = "The clipboard is not available";

fn invalid_time_message(text: &str) -> String {
    format!("{text} is not a valid time (HHMM, 0000–2359)")
}

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

    /// The value a pasted text gives this text field: a time becomes `HHMM`,
    /// other fields get the text on one line. `None` for an invalid time.
    fn pasted_value(self, text: &str) -> Option<String> {
        if self.is_time() {
            TimeOfDay::parse_pasted(text).map(TimeOfDay::format_hhmm)
        } else {
            Some(text.lines().map(str::trim).collect::<Vec<_>>().join(" "))
        }
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

impl FieldFocus {
    /// From of the first period: the focus after switching days.
    const START: FieldFocus = FieldFocus {
        period: 0,
        field: Field::FIRST,
    };
}

pub struct App {
    timesheet: Timesheet,
    storage: Storage,
    clipboard: Box<dyn Clipboard>,
    /// Whether the timesheet has changes that aren't saved yet.
    has_unsaved_changes: bool,
    /// Set when "Save" was chosen in the "Save before quitting?" popup.
    quit_after_saving: bool,
    /// Where today comes from; injected so tests can fake it.
    clock: Box<dyn Clock>,
    /// The clock's date as last read, so a frame or key press uses one date
    /// throughout; read again before each frame and each key press.
    today: Date,
    screen: Screen,
    selected: Date,
    /// A field of the right panel always has focus.
    focus: FieldFocus,
    /// The focused field is being edited while this is set.
    active_input: Option<ActiveInput>,
    /// A one-line notice in the right panel; cleared at the next key press.
    message: Option<String>,
    /// Periods of the selected day that failed the consistency check; cleared
    /// at the next key press, like the message.
    inconsistent_periods: Vec<usize>,
    /// Undo/redo history of accepted changes.
    history: History,
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
    /// Loads the timesheet at `path` and starts on the clock's today.
    pub fn open(
        fs: Box<dyn FileSystem>,
        path: PathBuf,
        clock: Box<dyn Clock>,
    ) -> Result<Self, LoadError> {
        let (timesheet, fingerprint) = store::load(fs.as_ref(), &path)?;
        Ok(Self::new(
            timesheet,
            Storage {
                fs,
                path,
                fingerprint,
                backup_made: false,
            },
            clock,
        ))
    }

    fn new(timesheet: Timesheet, storage: Storage, clock: Box<dyn Clock>) -> Self {
        let today = clock.today();
        Self {
            timesheet,
            storage,
            clipboard: Box::new(NoClipboard),
            has_unsaved_changes: false,
            quit_after_saving: false,
            clock,
            today,
            screen: Screen::Tracking,
            selected: calendar::start_day(today),
            focus: FieldFocus::START,
            active_input: None,
            message: None,
            inconsistent_periods: Vec::new(),
            history: History::default(),
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

    pub fn selected_day(&self) -> Date {
        self.selected
    }

    pub fn focus(&self) -> FieldFocus {
        self.focus
    }

    pub fn is_editing(&self) -> bool {
        self.active_input.is_some()
    }

    pub fn timesheet(&self) -> &Timesheet {
        &self.timesheet
    }

    fn is_tracking(&self) -> bool {
        self.screen == Screen::Tracking
    }

    /// The selected day as the right panel shows it: a day without periods
    /// shows one empty period, which only becomes a real period once
    /// something is filled in.
    fn shown_day(&self) -> Cow<'_, Day> {
        self.timesheet.day(self.selected).as_shown()
    }

    /// Whether the selected day shows the empty period of a day without periods.
    fn shows_empty_period(&self) -> bool {
        self.timesheet.day(self.selected).shows_empty_period()
    }

    /// Uses `clipboard` for Ctrl+C/Ctrl+V; without one, they do nothing.
    pub fn set_clipboard(&mut self, clipboard: Box<dyn Clipboard>) {
        self.clipboard = clipboard;
    }

    /// Reads today from the clock again, so balances, warnings and checks
    /// follow a date change; the selected day stays.
    pub fn refresh_today(&mut self) {
        self.today = self.clock.today();
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.has_unsaved_changes
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        // The date may have changed while waiting for this key.
        self.refresh_today();
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
            && self.is_tracking()
            && let Some(shortcut) = PeriodShortcut::for_key(key.code)
        {
            // Accept active input first; if that fails, the shortcut does nothing.
            if self.active_input.is_none() || self.accept() {
                self.run_period_shortcut(shortcut);
            }
            return Outcome::Continue;
        }
        if ctrl && self.is_tracking() {
            match key.code {
                KeyCode::Char('c') => {
                    self.copy();
                    return Outcome::Continue;
                }
                KeyCode::Char('v') => {
                    self.paste();
                    return Outcome::Continue;
                }
                _ => {}
            }
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
        if ctrl && key.code == KeyCode::Char('z') && self.screen == Screen::Tracking {
            self.undo();
            return Outcome::Continue;
        }
        if ctrl && key.code == KeyCode::Char('y') && self.screen == Screen::Tracking {
            self.redo();
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
            (Screen::Tracking, code) => self.handle_tracking_key(code, ctrl, alt),
        }
        Outcome::Continue
    }

    /// Keys of the Tracking screen while not editing. Focus is always in the
    /// right panel; the left panel has no keys of its own.
    fn handle_tracking_key(&mut self, code: KeyCode, ctrl: bool, alt: bool) {
        let plain = !ctrl && !alt;
        let ctrl_only = ctrl && !alt;
        match code {
            KeyCode::Up if ctrl_only => {
                self.leave_day_for(calendar::previous_weekday(self.selected))
            }
            KeyCode::Down if ctrl_only => self.leave_day_for(calendar::next_weekday(self.selected)),
            KeyCode::PageUp if ctrl_only => {
                self.leave_day_for(calendar::previous_week(self.selected))
            }
            KeyCode::PageDown if ctrl_only => {
                self.leave_day_for(calendar::next_week(self.selected))
            }
            KeyCode::Left if plain => self.move_focus_within_period(Direction::Backward),
            KeyCode::Right if plain => self.move_focus_within_period(Direction::Forward),
            KeyCode::Up if plain => self.move_focus_vertically(Direction::Backward),
            KeyCode::Down if plain => self.move_focus_vertically(Direction::Forward),
            KeyCode::Tab => self.move_focus_horizontally(Direction::Forward),
            KeyCode::BackTab => self.move_focus_horizontally(Direction::Backward),
            KeyCode::Enter if alt => self.add_period(),
            KeyCode::Enter => self.edit_or_toggle_focused_field(),
            // The empty period of a day without periods can't be deleted.
            KeyCode::Delete if alt && !self.shows_empty_period() => {
                let label = self.focused_period().map(Period::label).unwrap_or_default();
                self.popup = Some(Popup::delete_period(&label));
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
        let field = self.focus.field;
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

    /// The focused period: `None` on the empty period of a day without periods.
    fn focused_period(&self) -> Option<&Period> {
        self.timesheet
            .day(self.selected)
            .periods
            .get(self.focus.period)
    }

    /// Space toggles a focused checkbox; otherwise typing starts editing.
    fn edit_or_toggle_focused_field(&mut self) {
        match self.focus.field.period_type() {
            Some(period_type) => self.toggle_period_type(period_type),
            None => self.active_input = Some(ActiveInput::with_text(&self.focused_text())),
        }
    }

    /// Ctrl+C: copies the focused field's value, or the active input while
    /// editing. Checkboxes can't be copied.
    fn copy(&mut self) {
        if !self.focus.field.takes_text() {
            return;
        }
        let text = match &self.active_input {
            Some(input) => input.text(),
            None => self.focused_text(),
        };
        if !self.clipboard.set_text(&text) {
            self.message = Some(NO_CLIPBOARD.to_owned());
        }
    }

    /// Ctrl+V: puts the clipboard text into the focused field as accepted
    /// input, replacing any active input. A time may be pasted as `HHMM`,
    /// `HH:MM` or `H:MM`; anything else is rejected with a message.
    fn paste(&mut self) {
        let focus = self.focus;
        if !focus.field.takes_text() {
            return;
        }
        let Some(text) = self.clipboard.get_text() else {
            self.message = Some(NO_CLIPBOARD.to_owned());
            return;
        };
        let text = text.trim();
        if text.is_empty() {
            self.message = Some("The clipboard is empty".to_owned());
            return;
        }
        let Some(value) = focus.field.pasted_value(text) else {
            self.message = Some(invalid_time_message(text));
            return;
        };
        let previous = self.active_input.replace(ActiveInput::with_text(&value));
        if !self.accept() {
            self.active_input = previous;
        }
    }

    /// The focused text field's current value, as it is shown.
    fn focused_text(&self) -> String {
        let Some(period) = self.focused_period() else {
            return String::new();
        };
        let time = |t: Option<TimeOfDay>| t.map(TimeOfDay::format_hhmm).unwrap_or_default();
        match self.focus.field {
            Field::From => time(period.from),
            Field::To => time(period.to),
            Field::Project => period.project.clone().unwrap_or_default(),
            Field::Notes => period.notes.clone().unwrap_or_default(),
            Field::Break | Field::Vacation | Field::Sick | Field::Holiday => String::new(),
        }
    }

    fn toggle_period_type(&mut self, period_type: PeriodType) {
        let period = self.focus.period;
        self.edit_selected_day(|day| day.toggle_period_type(period, period_type));
    }

    fn run_period_shortcut(&mut self, shortcut: PeriodShortcut) {
        match shortcut {
            PeriodShortcut::Toggle(period_type) => self.toggle_period_type(period_type),
            PeriodShortcut::Focus(field) => self.focus.field = field,
        }
    }

    /// Typing in a focused text field replaces its value with active input.
    fn start_editing(&mut self, c: char) {
        let field = self.focus.field;
        if !field.accepts(c, 0) {
            return;
        }
        // Notes keeps its text; other fields are replaced by what you type.
        let mut input = if field == Field::Notes {
            ActiveInput::with_text(&self.focused_text())
        } else {
            ActiveInput::with_text("")
        };
        input.insert(c);
        self.active_input = Some(input);
    }

    /// Stores the active input in the focused field. An invalid time keeps the
    /// active input and shows a message. Returns whether it was accepted.
    fn accept(&mut self) -> bool {
        let focus = self.focus;
        let Some(input) = &self.active_input else {
            return false;
        };
        let text = input.text();
        let time = if focus.field.is_time() && !text.is_empty() {
            match TimeOfDay::parse_hhmm(&text) {
                Some(time) => Some(time),
                None => {
                    self.message = Some(invalid_time_message(&text));
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

    /// Changes the selected day as an accepted change: recorded for undo and
    /// tracked as unsaved, but only when something did change. Undo/redo and
    /// sorting change days without going through here. On a day without
    /// periods, `edit` sees the empty period the day shows; it only becomes a
    /// real period if `edit` fills something in. Periods emptied later stay
    /// until the day is left (they are never saved).
    fn edit_selected_day<R>(&mut self, edit: impl FnOnce(&mut Day) -> R) -> R {
        let day = self.timesheet.day_mut(self.selected);
        let before = day.clone();
        let showed_empty_period = day.shows_empty_period();
        if showed_empty_period {
            day.periods.push(Period::default());
        }
        let result = edit(day);
        if showed_empty_period && day.is_empty() {
            day.periods.clear();
        }
        if *day != before {
            let after = day.clone();
            self.history.record(Change {
                date: self.selected,
                before,
                after,
            });
            self.mark_changed(self.selected);
        }
        result
    }

    fn mark_changed(&mut self, date: Date) {
        self.has_unsaved_changes = true;
        self.changed_days.insert(date);
    }

    /// Ctrl+Z: undoes the last change, selecting its day.
    fn undo(&mut self) {
        let restored = self.history.undo().map(|c| (c.date, c.before.clone()));
        self.restore_or_say(restored, "Nothing to undo");
    }

    /// Ctrl+Y: redoes the last undone change, selecting its day.
    fn redo(&mut self) {
        let restored = self.history.redo().map(|c| (c.date, c.after.clone()));
        self.restore_or_say(restored, "Nothing to redo");
    }

    fn restore_or_say(&mut self, restored: Option<(Date, Day)>, nothing: &str) {
        match restored {
            Some((date, day)) => self.restore_day(date, day),
            None => self.message = Some(nothing.to_owned()),
        }
    }

    /// Puts `day` back on `date` and selects it. On the same day, focus stays
    /// at the same position if that period exists, else moves to the same
    /// field of the last period; another day starts on From of its first period.
    fn restore_day(&mut self, date: Date, day: Day) {
        self.timesheet.set_day(date, day);
        self.mark_changed(date);
        if date == self.selected {
            self.focus.period = self.focus.period.min(self.period_count() - 1);
        } else {
            self.select_day(date);
        }
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

    /// Sorts the selected day and removes its empty periods. Not an undo step.
    fn normalize_selected_day(&mut self) {
        let moved = self.timesheet.day_mut(self.selected).normalize();
        self.follow_moved_periods(&moved);
    }

    /// Keeps focus on the same period after the selected day was sorted. If
    /// that period was removed, focus moves to From of the first period.
    fn follow_moved_periods(&mut self, moved: &[Option<usize>]) {
        self.focus = match moved.get(self.focus.period).copied().flatten() {
            Some(period) => FieldFocus {
                period,
                ..self.focus
            },
            None => FieldFocus::START,
        };
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
            self.show_problem(inconsistent.problem);
        }
    }

    /// Alt+Enter adds a period after the last one and focuses its From,
    /// unless the last period is still open (which includes an empty period,
    /// such as the one a day without periods shows).
    fn add_period(&mut self) {
        let last_is_open = self.shown_day().periods.last().is_some_and(Period::is_open);
        if last_is_open {
            return;
        }
        let period = self.edit_selected_day(Day::add_period);
        self.focus = FieldFocus {
            period,
            field: Field::From,
        };
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
    /// After deleting the only period, the day shows an empty period, with
    /// focus on its From.
    fn delete_focused_period(&mut self) {
        let focus = self.focus;
        let count = self.edit_selected_day(|day| {
            day.remove_period(focus.period);
            day.periods.len()
        });
        self.focus = if count == 0 {
            FieldFocus::START
        } else {
            FieldFocus {
                period: focus.period.min(count - 1),
                ..focus
            }
        };
    }

    /// Selects another day, focusing From of its first period.
    fn select_day(&mut self, date: Date) {
        self.selected = date;
        self.focus = FieldFocus::START;
        self.first_visible_period_row = 0;
    }

    /// The number of periods the right panel shows: at least one.
    fn period_count(&self) -> usize {
        self.shown_day().periods.len()
    }

    /// ↑/↓: the same field in the previous/next period, stopping at the first
    /// and last period.
    fn move_focus_vertically(&mut self, direction: Direction) {
        let period = self.focus.period;
        self.focus.period = match direction {
            Direction::Forward => (period + 1).min(self.period_count() - 1),
            Direction::Backward => period.saturating_sub(1),
        };
    }

    /// ←/→: the previous/next field of the focused period, stopping at its
    /// first and last field.
    fn move_focus_within_period(&mut self, direction: Direction) {
        let field = match direction {
            Direction::Forward => self.focus.field.next(),
            Direction::Backward => self.focus.field.previous(),
        };
        if let Some(field) = field {
            self.focus.field = field;
        }
    }

    /// Tab/Shift-Tab: the next/previous field, continuing in the next/previous
    /// period. Focus stays put at the very first and last field.
    fn move_focus_horizontally(&mut self, direction: Direction) {
        let count = self.period_count();
        let focus = self.focus;
        self.focus = match direction {
            Direction::Forward => match focus.field.next() {
                Some(field) => FieldFocus { field, ..focus },
                None if focus.period + 1 < count => FieldFocus {
                    period: focus.period + 1,
                    field: Field::FIRST,
                },
                None => focus,
            },
            Direction::Backward => match focus.field.previous() {
                Some(field) => FieldFocus { field, ..focus },
                None if focus.period > 0 => FieldFocus {
                    period: focus.period - 1,
                    field: Field::LAST,
                },
                None => focus,
            },
        };
    }
}
