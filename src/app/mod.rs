//! Interactive application state and key handling. Drawing lives in `render`.

mod active_input;
mod dialog;
mod go_to_date;
mod history;
mod render;

use std::{borrow::Cow, collections::HashSet, path::PathBuf};

use jiff::civil::Date;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use active_input::ActiveInput;
use dialog::{Dialog, DialogAction};
use go_to_date::{DATE_FORMATS, GoToDate, GoToDateAction};
use history::{Change, History};

use crate::{
    clipboard::{Clipboard, NoClipboard},
    clock::Clock,
    domain::{
        ConsistencyProblem, Day, Extent, ExtentKind, InconsistentDay, PartOfDay, Period,
        PeriodType, TimeOfDay, Timesheet, calendar,
    },
    store::{self, Backup, FileSystem, Fingerprint, LoadError, Overwrite, SaveError},
};

const NO_CLIPBOARD: &str = "The clipboard is not available";

/// The number of periods PgUp/PgDn move focus.
const PAGE_PERIODS: usize = 10;

/// The number of periods ↑/↓ move focus.
const ARROW_PERIODS: usize = 1;

fn invalid_time_message(text: &str) -> String {
    format!("{text} is not a valid time (HHMM, 0000–2359)")
}

fn invalid_date_message(text: &str) -> String {
    if text.is_empty() {
        format!("Type a date ({DATE_FORMATS})")
    } else {
        format!("{text} is not a valid date ({DATE_FORMATS})")
    }
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
    /// Shown instead of From and To in a period with a part of day.
    PartOfDay,
    Project,
    Notes,
    Break,
    Free,
    Vacation,
    Sick,
    Holiday,
}

impl Field {
    /// All fields in Tab order, which is their order on screen; a period has
    /// either From and To or the part-of-day field.
    const ALL: [Field; 10] = [
        Field::From,
        Field::To,
        Field::PartOfDay,
        Field::Project,
        Field::Break,
        Field::Free,
        Field::Vacation,
        Field::Sick,
        Field::Holiday,
        Field::Notes,
    ];
    const FIRST: Field = Field::ALL[0];
    const LAST: Field = Field::ALL[Field::ALL.len() - 1];

    /// The fields of a period whose extent is of `kind`, in Tab order: From
    /// and To, or the part-of-day field instead.
    fn of_period(kind: ExtentKind) -> Vec<Field> {
        Self::ALL
            .into_iter()
            .filter(|&field| field.in_period(kind) == field)
            .collect()
    }

    /// The field taking this one's place in a period whose extent is of
    /// `kind`: the part-of-day field stands for both From and To, and From
    /// for the part-of-day field.
    fn in_period(self, kind: ExtentKind) -> Field {
        match (self, kind) {
            (Field::From | Field::To, ExtentKind::PartOfDay) => Field::PartOfDay,
            (Field::PartOfDay, ExtentKind::FromTo) => Field::From,
            _ => self,
        }
    }

    /// The next or previous field, in `direction`, of a period whose extent
    /// is of `kind`; `None` at its last or first field.
    fn neighbor(self, kind: ExtentKind, direction: Direction) -> Option<Field> {
        let fields = Self::of_period(kind);
        let index = fields.iter().position(|&f| f == self)?;
        let neighbor = match direction {
            Direction::Forward => Some(index + 1),
            Direction::Backward => index.checked_sub(1),
        };
        neighbor.and_then(|i| fields.get(i).copied())
    }

    /// From and To take times typed as HHMM.
    fn is_time(self) -> bool {
        matches!(self, Field::From | Field::To)
    }

    /// The period-type checkboxes, in Tab order.
    pub const CHECKBOXES: [Field; 5] = [
        Field::Break,
        Field::Free,
        Field::Vacation,
        Field::Sick,
        Field::Holiday,
    ];

    /// The period type a checkbox field stands for.
    pub fn period_type(self) -> Option<PeriodType> {
        match self {
            Field::Break => Some(PeriodType::Break),
            Field::Free => Some(PeriodType::Free),
            Field::Vacation => Some(PeriodType::Vacation),
            Field::Sick => Some(PeriodType::Sick),
            Field::Holiday => Some(PeriodType::Holiday),
            Field::From | Field::To | Field::PartOfDay | Field::Project | Field::Notes => None,
        }
    }

    /// Fields that take typed text: every field that isn't a checkbox or the
    /// part-of-day field.
    fn takes_text(self) -> bool {
        self.period_type().is_none() && self != Field::PartOfDay
    }
}

/// Pasted text on one line: line breaks become spaces.
fn one_line(text: &str) -> String {
    text.lines().map(str::trim).collect::<Vec<_>>().join(" ")
}

/// What a period-definition key does to the focused period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeriodDefinition {
    /// Alt+b/f/s/v/h: toggles Break/Free/Sick/Vacation/Holiday.
    ToggleType(PeriodType),
    /// Alt+0: from–to with empty times; Alt+1/2/3: full day, 1st half or
    /// 2nd half.
    SwitchExtent(Extent),
}

impl PeriodDefinition {
    /// The period definition of Alt and `code`, if any.
    fn of_alt_key(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::Char('b') => Some(Self::ToggleType(PeriodType::Break)),
            KeyCode::Char('f') => Some(Self::ToggleType(PeriodType::Free)),
            KeyCode::Char('v') => Some(Self::ToggleType(PeriodType::Vacation)),
            KeyCode::Char('s') => Some(Self::ToggleType(PeriodType::Sick)),
            KeyCode::Char('h') => Some(Self::ToggleType(PeriodType::Holiday)),
            KeyCode::Char('0') => Some(Self::SwitchExtent(Extent::default())),
            KeyCode::Char(digit @ '1'..='3') => {
                let index = usize::from(digit as u8 - b'1');
                Some(Self::SwitchExtent(Extent::PartOfDay(PartOfDay::ALL[index])))
            }
            _ => None,
        }
    }
}

/// What a key does while editing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EditingAction {
    /// Esc: rejects the active input.
    Reject,
    /// Accepts the active input, first taking the project suggestion if
    /// `takes_suggestion`, then moves focus `then`.
    Accept {
        takes_suggestion: bool,
        then: AfterAccepting,
    },
    /// →: takes the project suggestion, or else moves the cursor.
    TakeSuggestionOrEdit,
    /// Types, moves the cursor or deletes; other keys do nothing.
    Edit,
}

impl EditingAction {
    fn of(key: KeyEvent) -> Self {
        let plain = Modifiers::of(key) == Modifiers::None;
        let ctrl_only = Modifiers::of(key) == Modifiers::Ctrl;
        let accept = |then| EditingAction::Accept {
            takes_suggestion: then != AfterAccepting::Stay,
            then,
        };
        match key.code {
            KeyCode::Esc => EditingAction::Reject,
            KeyCode::Enter if plain => accept(AfterAccepting::Stay),
            KeyCode::Enter if ctrl_only => accept(AfterAccepting::NextPeriod),
            KeyCode::Tab if plain => accept(AfterAccepting::Field(Direction::Forward)),
            KeyCode::BackTab if plain => accept(AfterAccepting::Field(Direction::Backward)),
            // Unlike Modifiers, Shift counts here: Shift+→ only moves the cursor.
            KeyCode::Right if key.modifiers.is_empty() => EditingAction::TakeSuggestionOrEdit,
            _ => EditingAction::Edit,
        }
    }
}

/// Where focus goes after accepting the active input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AfterAccepting {
    /// Enter: stays on the field.
    Stay,
    /// Tab/Shift-Tab: the next/previous field.
    Field(Direction),
    /// Ctrl+Enter: From of the next period.
    NextPeriod,
}

/// Which way focus moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Backward,
    Forward,
}

/// The Ctrl and Alt modifiers a key was pressed with; Shift doesn't count.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Modifiers {
    None,
    Ctrl,
    /// Alt, or Ctrl+Alt.
    Alt,
}

impl Modifiers {
    fn of(key: KeyEvent) -> Self {
        if key.modifiers.contains(KeyModifiers::ALT) {
            Modifiers::Alt
        } else if key.modifiers.contains(KeyModifiers::CONTROL) {
            Modifiers::Ctrl
        } else {
            Modifiers::None
        }
    }
}

/// The focused field of the focused period in the right panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldFocus {
    pub period: usize,
    pub field: Field,
}

/// What has focus in the right panel: the day note or a field of a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// The day note, below the last period.
    DayNote,
    Period(FieldFocus),
}

impl Focus {
    /// From of the first period: the focus after switching days.
    const START: Focus = Focus::Period(FieldFocus {
        period: 0,
        field: Field::FIRST,
    });

    /// The focused field of a period; `None` on the day note.
    pub fn field(self) -> Option<Field> {
        match self {
            Focus::DayNote => None,
            Focus::Period(focus) => Some(focus.field),
        }
    }

    /// The index of the focused period; `None` on the day note.
    pub fn period(self) -> Option<usize> {
        match self {
            Focus::DayNote => None,
            Focus::Period(focus) => Some(focus.period),
        }
    }

    /// From and To take times typed as HHMM.
    fn is_time(self) -> bool {
        self.field().is_some_and(Field::is_time)
    }

    /// Whether the focus is on an input field: the day note or a period
    /// field that isn't a checkbox.
    fn takes_text(self) -> bool {
        self.field().is_none_or(Field::takes_text)
    }

    /// The value a pasted text gives this input field: a time becomes `HHMM`,
    /// other fields get the text on one line. `None` for an invalid time.
    fn pasted_value(self, text: &str) -> Option<String> {
        if self.is_time() {
            TimeOfDay::parse_pasted(text).map(TimeOfDay::format_hhmm)
        } else {
            Some(one_line(text))
        }
    }

    /// Whether typing `c` adds it to an active input of `len` characters:
    /// time fields take at most four digits, other input fields anything.
    fn accepts(self, c: char, len: usize) -> bool {
        self.takes_text() && (!self.is_time() || (c.is_ascii_digit() && len < 4))
    }
}

pub struct App {
    timesheet: Timesheet,
    storage: Storage,
    clipboard: Box<dyn Clipboard>,
    /// Whether the timesheet has changes that aren't saved yet.
    has_unsaved_changes: bool,
    /// Set when "Save" was chosen in the "Save before quitting?" dialog.
    quit_after_saving: bool,
    /// Where today comes from; injected so tests can fake it.
    clock: Box<dyn Clock>,
    /// The clock's date as last read, so a frame or key press uses one date
    /// throughout; read again before each frame and each key press.
    today: Date,
    screen: Screen,
    selected: Date,
    /// A field of the right panel always has focus.
    focus: Focus,
    /// The focused field is being edited while this is set.
    active_input: Option<ActiveInput>,
    /// A global notification: a one-line message centered over the screen;
    /// cleared at the next key press.
    notification: Option<String>,
    /// Periods of the selected day that failed the consistency check; cleared
    /// at the next key press, like the notification.
    inconsistent_periods: Vec<usize>,
    /// Undo/redo history of accepted changes.
    history: History,
    /// Days changed in this session; only these are checked when leaving them.
    changed_days: HashSet<Date>,
    /// The day named in a consistency-check dialog, for "Go to day".
    inconsistent_day: Option<InconsistentDay>,
    dialog: Option<Dialog>,
    /// The "Go to date" dialog, while it is open.
    go_to_date: Option<GoToDate>,
    /// `None` until the first render.
    first_visible_day: Option<Date>,
    /// The last day the day list showed at the last render.
    last_visible_day: Option<Date>,
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
        let mut app = Self {
            timesheet,
            storage,
            clipboard: Box::new(NoClipboard),
            has_unsaved_changes: false,
            quit_after_saving: false,
            clock,
            today,
            screen: Screen::Tracking,
            selected: calendar::start_day(today),
            focus: Focus::START,
            active_input: None,
            notification: None,
            inconsistent_periods: Vec::new(),
            history: History::default(),
            changed_days: HashSet::new(),
            inconsistent_day: None,
            dialog: None,
            go_to_date: None,
            first_visible_day: None,
            last_visible_day: None,
            first_visible_period_row: 0,
        };
        app.fit_focus_to_period();
        app
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn selected_day(&self) -> Date {
        self.selected
    }

    pub fn focus(&self) -> Focus {
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

    /// Handles a key press. Afterwards, focus is on a field the focused
    /// period has: whatever the key changed, a part-of-day period has the
    /// part-of-day field instead of From and To.
    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        let outcome = self.dispatch_key(key);
        self.fit_focus_to_period();
        outcome
    }

    /// Moves focus from From or To to the part-of-day field if the focused
    /// period has a part of day, and from the part-of-day field to From if
    /// it hasn't.
    fn fit_focus_to_period(&mut self) {
        if let Focus::Period(focus) = self.focus {
            let field = focus.field.in_period(self.extent_kind(focus.period));
            self.focus = Focus::Period(FieldFocus { field, ..focus });
        }
    }

    /// Whether the period at `period` of the selected day, as shown, is
    /// from–to or a part of day.
    fn extent_kind(&self, period: usize) -> ExtentKind {
        self.shown_day()
            .periods
            .get(period)
            .map(|p| p.extent.kind())
            .unwrap_or_default()
    }

    fn dispatch_key(&mut self, key: KeyEvent) -> Outcome {
        // The date may have changed while waiting for this key.
        self.refresh_today();
        let modifiers = Modifiers::of(key);
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        self.notification = None;
        self.inconsistent_periods.clear();
        if let Some(dialog) = &mut self.dialog {
            if let Some(action) = dialog.handle_key(key.code) {
                self.dialog = None;
                return self.run_dialog_action(action);
            }
            return Outcome::Continue;
        }
        if self.go_to_date.is_some() {
            self.handle_go_to_date_key(key);
            return Outcome::Continue;
        }
        // The day note isn't in a period, so period definitions don't apply to it.
        if alt
            && self.is_tracking()
            && let Some(period) = self.focus.period()
            && let Some(definition) = PeriodDefinition::of_alt_key(key.code)
        {
            // Accept active input first; if that fails, the shortcut does nothing.
            if self.active_input.is_none() || self.accept() {
                match definition {
                    PeriodDefinition::ToggleType(period_type) => {
                        self.toggle_period_type(period, period_type)
                    }
                    PeriodDefinition::SwitchExtent(extent) => self.switch_extent(period, extent),
                }
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
            if self.check_all_days(("Quit anyway", DialogAction::QuitAnyway)) {
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
            if self.check_all_days(("Save anyway", DialogAction::SaveAnyway)) {
                return self.save(Overwrite::IfUnchanged);
            }
            return Outcome::Continue;
        }
        match (self.screen, key.code) {
            (_, KeyCode::F(1)) => self.screen = Screen::Help,
            (_, KeyCode::F(2)) | (Screen::Help, KeyCode::Esc) => self.screen = Screen::Tracking,
            (Screen::Help, _) => {}
            (Screen::Tracking, code) => self.handle_tracking_key(code, modifiers),
        }
        Outcome::Continue
    }

    /// Keys of the Tracking screen while not editing. Focus is always in the
    /// right panel; the left panel has no keys of its own.
    fn handle_tracking_key(&mut self, code: KeyCode, modifiers: Modifiers) {
        let plain = modifiers == Modifiers::None;
        let ctrl_only = modifiers == Modifiers::Ctrl;
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
            KeyCode::Char('g') if ctrl_only => self.go_to_date = Some(GoToDate::new()),
            KeyCode::Left if plain => self.move_focus_within_period(Direction::Backward),
            KeyCode::Right if plain => self.move_focus_within_period(Direction::Forward),
            KeyCode::Up if plain => self.move_focus_vertically(Direction::Backward, ARROW_PERIODS),
            KeyCode::Down if plain => self.move_focus_vertically(Direction::Forward, ARROW_PERIODS),
            KeyCode::PageUp if plain => {
                self.move_focus_vertically(Direction::Backward, PAGE_PERIODS)
            }
            KeyCode::PageDown if plain => {
                self.move_focus_vertically(Direction::Forward, PAGE_PERIODS)
            }
            KeyCode::Home if plain => self.move_focus_to_field(Field::FIRST),
            KeyCode::End if plain => self.move_focus_to_field(Field::LAST),
            KeyCode::Tab if plain => self.move_focus_in_tab_order(Direction::Forward),
            KeyCode::BackTab if plain => self.move_focus_in_tab_order(Direction::Backward),
            KeyCode::Enter if ctrl_only => self.focus_next_period(),
            KeyCode::Enter if plain => self.edit_or_toggle_focused_field(),
            // The empty period of a day without periods can't be deleted, and
            // the day note isn't a period.
            KeyCode::Delete
                if ctrl_only && !self.shows_empty_period() && self.focus.period().is_some() =>
            {
                let label = self.focused_period().map(Period::label).unwrap_or_default();
                self.dialog = Some(Dialog::delete_period(&label));
            }
            KeyCode::Delete if plain => self.clear_focused_field(),
            KeyCode::Char(c) if plain => self.start_editing(c),
            _ => {}
        }
    }

    /// While editing, only editing keys work; Enter, Tab/Shift-Tab and
    /// Ctrl+Enter accept, Esc rejects. → at the end of the input takes the
    /// project suggestion and keeps editing; Tab/Shift-Tab and Ctrl+Enter
    /// take it before accepting. Other keys are ignored silently.
    fn handle_editing_key(&mut self, key: KeyEvent) {
        match EditingAction::of(key) {
            EditingAction::Reject => self.active_input = None,
            EditingAction::Accept {
                takes_suggestion,
                then,
            } => {
                if takes_suggestion {
                    self.take_project_suggestion();
                }
                if self.accept() {
                    self.move_after_accepting(then);
                }
            }
            EditingAction::TakeSuggestionOrEdit => {
                if !self.take_project_suggestion() {
                    self.edit_active_input(key);
                }
            }
            EditingAction::Edit => self.edit_active_input(key),
        }
    }

    /// Applies an editing key or types a character the focused field takes.
    fn edit_active_input(&mut self, key: KeyEvent) {
        let focus = self.focus;
        if let Some(input) = &mut self.active_input
            && !input.apply_editing_key(key)
            && let KeyCode::Char(c) = key.code
            && Modifiers::of(key) == Modifiers::None
            && focus.accepts(c, input.len())
        {
            input.insert(c);
        }
    }

    /// Moves focus after the active input was accepted.
    fn move_after_accepting(&mut self, then: AfterAccepting) {
        match then {
            AfterAccepting::Stay => {}
            AfterAccepting::Field(direction) => self.move_focus_in_tab_order(direction),
            AfterAccepting::NextPeriod => self.focus_next_period(),
        }
    }

    /// Fills in the project suggestion, if there is one, keeping the cursor
    /// at the end. Returns whether there was one.
    fn take_project_suggestion(&mut self) -> bool {
        let suggestion = self.project_suggestion();
        if let (Some(project), Some(input)) = (&suggestion, &mut self.active_input) {
            input.replace_with(project);
        }
        suggestion.is_some()
    }

    /// The earlier project suggested while editing Project with the cursor at
    /// the end of the active input, once something was typed or deleted in
    /// it; shown as ghost text, taken with →, Tab/Shift-Tab and Ctrl+Enter.
    /// Owned, so the active input can change while it is held.
    fn project_suggestion(&self) -> Option<String> {
        self.active_input
            .as_ref()
            .filter(|input| {
                self.focus.field() == Some(Field::Project)
                    && input.is_cursor_at_end()
                    && input.is_changed()
            })
            .and_then(|input| self.timesheet.project_suggestion(&input.text()))
            .map(str::to_owned)
    }

    /// The focused period: `None` on the empty period of a day without
    /// periods and on the day note.
    fn focused_period(&self) -> Option<&Period> {
        let periods = &self.timesheet.day(self.selected).periods;
        self.focus.period().and_then(|period| periods.get(period))
    }

    /// The focused checkbox, as its period and period type; `None` on input
    /// fields.
    fn focused_checkbox(&self) -> Option<(usize, PeriodType)> {
        match self.focus {
            Focus::Period(focus) => focus.field.period_type().map(|t| (focus.period, t)),
            Focus::DayNote => None,
        }
    }

    /// Enter (not editing): toggles a focused checkbox, or starts editing a
    /// focused input field with its current value. Does nothing on the
    /// part-of-day field.
    fn edit_or_toggle_focused_field(&mut self) {
        match self.focused_checkbox() {
            Some((period, period_type)) => self.toggle_period_type(period, period_type),
            None if self.focus.takes_text() => {
                self.active_input = Some(ActiveInput::with_text(&self.focused_text()));
            }
            None => {}
        }
    }

    /// Ctrl+C: copies the focused field's value, or the active input while
    /// editing. Checkboxes can't be copied.
    fn copy(&mut self) {
        if !self.focus.takes_text() {
            return;
        }
        let text = match &self.active_input {
            Some(input) => input.text(),
            None => self.focused_text(),
        };
        if !self.clipboard.set_text(&text) {
            self.notify(NO_CLIPBOARD);
        }
    }

    /// Ctrl+V: puts the clipboard text into the focused field as accepted
    /// input, replacing any active input. A time may be pasted as `HHMM`,
    /// `HH:MM` or `H:MM`; anything else is rejected with a notification.
    fn paste(&mut self) {
        let focus = self.focus;
        if !focus.takes_text() {
            return;
        }
        let Some(clipboard_text) = self.clipboard.get_text() else {
            self.notify(NO_CLIPBOARD);
            return;
        };
        let text = clipboard_text.trim();
        // Only spaces paste as no day note, as when typed; other fields take
        // them for an empty clipboard.
        if clipboard_text.is_empty() || (text.is_empty() && focus != Focus::DayNote) {
            self.notify("The clipboard is empty");
            return;
        }
        let Some(value) = focus.pasted_value(text) else {
            self.notify(invalid_time_message(text));
            return;
        };
        let previous = self.active_input.replace(ActiveInput::with_text(&value));
        if !self.accept() {
            self.active_input = previous;
        }
    }

    /// The clipboard's text, trimmed. `None`, with a notification, when the
    /// clipboard isn't available or is empty.
    fn clipboard_text(&mut self) -> Option<String> {
        let text = self.clipboard.get_text().map(|text| text.trim().to_owned());
        match text.as_deref() {
            None => self.notify(NO_CLIPBOARD),
            Some("") => self.notify("The clipboard is empty"),
            Some(_) => {}
        }
        text.filter(|text| !text.is_empty())
    }

    /// The focused input field's current value, as it is shown.
    fn focused_text(&self) -> String {
        match self.focus {
            Focus::DayNote => self.timesheet.day(self.selected).day_note.clone(),
            Focus::Period(focus) => self
                .focused_period()
                .and_then(|period| field_text(period, focus.field)),
        }
        .unwrap_or_default()
    }

    /// Checks or unchecks `period_type` on the period at `period`.
    fn toggle_period_type(&mut self, period: usize, period_type: PeriodType) {
        self.edit_selected_day(|day| day.toggle_period_type(period, period_type));
    }

    /// Switches the period at `period` to `extent`: a part of day, or
    /// from–to with empty times (only if it has a part of day). Focus moves
    /// to the part-of-day field, or to From after switching to from–to.
    fn switch_extent(&mut self, period: usize, extent: Extent) {
        let had_part_of_day = self.extent_kind(period) == ExtentKind::PartOfDay;
        self.edit_selected_day(|day| day.switch_extent(period, extent));
        let field = match extent.kind() {
            ExtentKind::PartOfDay => Some(Field::PartOfDay),
            ExtentKind::FromTo => had_part_of_day.then_some(Field::From),
        };
        if let Some(field) = field {
            self.focus = Focus::Period(FieldFocus { period, field });
        }
    }

    /// Typing in a focused input field replaces its value with active input.
    fn start_editing(&mut self, c: char) {
        if !self.focus.accepts(c, 0) {
            return;
        }
        let mut input = ActiveInput::with_text("");
        input.insert(c);
        self.active_input = Some(input);
    }

    /// Del (not editing): empties a input field, unchecks a checkbox or
    /// switches a part of day back to from–to, as an accepted change.
    fn clear_focused_field(&mut self) {
        let part_of_day_period = match self.focus {
            Focus::Period(focus) if focus.field == Field::PartOfDay => Some(focus.period),
            _ => None,
        };
        match (part_of_day_period, self.focused_checkbox()) {
            (Some(period), _) => self.switch_extent(period, Extent::default()),
            (None, Some((period, period_type))) => {
                let checked = self
                    .focused_period()
                    .is_some_and(|period| period.period_type == period_type);
                if checked {
                    self.toggle_period_type(period, period_type);
                }
            }
            (None, None) => {
                self.active_input = Some(ActiveInput::with_text(""));
                self.accept();
            }
        }
    }

    /// Stores the active input in the focused field. An invalid time keeps the
    /// active input and shows a notification. Returns whether it was accepted.
    fn accept(&mut self) -> bool {
        let focus = self.focus;
        let Some(input) = &self.active_input else {
            return false;
        };
        let text = input.text();
        let time = if focus.is_time() && !text.is_empty() {
            match TimeOfDay::parse_hhmm(&text) {
                Some(time) => Some(time),
                None => {
                    self.notify(invalid_time_message(&text));
                    return false;
                }
            }
        } else {
            None
        };
        self.edit_selected_day(|day| match focus {
            Focus::DayNote => day.set_day_note(&text),
            Focus::Period(focus) => {
                let value = (!text.is_empty()).then_some(text);
                match focus.field {
                    Field::From => day.set_from(focus.period, time),
                    Field::To => day.set_to(focus.period, time),
                    Field::Project => {
                        if let Some(period) = day.periods.get_mut(focus.period) {
                            period.project = value;
                        }
                    }
                    Field::Notes => {
                        if let Some(period) = day.periods.get_mut(focus.period) {
                            period.notes = value;
                        }
                    }
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
        self.restore_or_notify(restored, "Nothing to undo");
    }

    /// Ctrl+Y: redoes the last undone change, selecting its day.
    fn redo(&mut self) {
        let restored = self.history.redo().map(|c| (c.date, c.after.clone()));
        self.restore_or_notify(restored, "Nothing to redo");
    }

    /// Shows `text` as a global notification until the next key press.
    fn notify(&mut self, text: impl Into<String>) {
        self.notification = Some(text.into());
    }

    /// Restores an undone or redone day, or notifies `nothing_to_do` if there is none.
    fn restore_or_notify(&mut self, restored: Option<(Date, Day)>, nothing_to_do: &str) {
        match restored {
            Some((date, day)) => self.restore_day(date, day),
            None => self.notify(nothing_to_do),
        }
    }

    /// Puts `day` back on `date` and selects it. On the same day, focus stays
    /// at the same position if that period exists, else moves to the same
    /// field of the last period; another day starts on From of its first period.
    fn restore_day(&mut self, date: Date, day: Day) {
        self.timesheet.set_day(date, day);
        self.mark_changed(date);
        if date == self.selected {
            let last_period = self.period_count() - 1;
            if let Focus::Period(focus) = &mut self.focus {
                focus.period = focus.period.min(last_period);
            }
        } else {
            self.select_day(date);
        }
    }

    /// Selects `date`, unless the selected day was changed in this session
    /// and fails the consistency check: then it stays selected, with a
    /// notification and its offending periods highlighted. Either way, the
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

    /// Keys while the "Go to date" dialog is open. Ctrl+C/Ctrl+V copy and
    /// paste the input field's text while it has focus, as in other input
    /// fields; the dialog handles all other keys.
    fn handle_go_to_date_key(&mut self, key: KeyEvent) {
        match (key.code, Modifiers::of(key)) {
            (KeyCode::Char('c'), Modifiers::Ctrl) => self.copy_go_to_date_input(),
            (KeyCode::Char('v'), Modifiers::Ctrl) => self.paste_into_go_to_date_input(),
            _ => {
                let action = self.go_to_date.as_mut().and_then(|g| g.handle_key(key));
                if let Some(action) = action {
                    self.run_go_to_date_action(action);
                }
            }
        }
    }

    /// Ctrl+C in the "Go to date" dialog: copies the typed date.
    fn copy_go_to_date_input(&mut self) {
        let text = self
            .go_to_date
            .as_mut()
            .and_then(GoToDate::focused_input)
            .map(|input| input.text());
        if let Some(text) = text
            && !self.clipboard.set_text(&text)
        {
            self.notify(NO_CLIPBOARD);
        }
    }

    /// Ctrl+V in the "Go to date" dialog: replaces the typed date with the
    /// clipboard text, on one line. It doesn't go to the date yet.
    fn paste_into_go_to_date_input(&mut self) {
        let input_focused = self
            .go_to_date
            .as_mut()
            .is_some_and(|g| g.focused_input().is_some());
        if let Some(text) = input_focused.then(|| self.clipboard_text()).flatten()
            && let Some(input) = self.go_to_date.as_mut().and_then(GoToDate::focused_input)
        {
            *input = ActiveInput::with_text(&one_line(&text));
        }
    }

    /// Runs the choice made in the "Go to date" dialog. A typed date on a
    /// weekend, like today on a weekend, means the Friday before. An invalid
    /// date keeps the dialog open, with a notification, so it can be fixed.
    fn run_go_to_date_action(&mut self, action: GoToDateAction) {
        let date = match action {
            GoToDateAction::Go(text) => {
                let date = calendar::parse_date(&text, self.today);
                if date.is_none() {
                    self.notify(invalid_date_message(text.trim()));
                }
                date
            }
            GoToDateAction::Today => Some(self.today),
            GoToDateAction::Cancel => {
                self.go_to_date = None;
                None
            }
        };
        if let Some(date) = date {
            self.go_to_date = None;
            self.go_to_day(calendar::weekday_on_or_before(date));
        }
    }

    /// Leaves the selected day for `date` like other day navigation. If
    /// `date` isn't in the day list as last shown, the list is laid out
    /// afresh around it, as on startup, instead of scrolling there.
    fn go_to_day(&mut self, date: Date) {
        let visible = self
            .first_visible_day
            .zip(self.last_visible_day)
            .is_some_and(|(first, last)| (first..=last).contains(&date));
        self.leave_day_for(date);
        if !visible && self.selected == date {
            self.first_visible_day = None;
        }
    }

    /// Sorts the selected day and removes its empty periods. Not an undo step.
    fn normalize_selected_day(&mut self) {
        let moved = self.timesheet.day_mut(self.selected).normalize();
        self.follow_moved_periods(&moved);
    }

    /// Keeps focus on the same period after the selected day was sorted. If
    /// that period was removed, focus moves to From of the first period. The
    /// day note keeps focus.
    fn follow_moved_periods(&mut self, moved: &[Option<usize>]) {
        self.focus = match self.focus {
            Focus::DayNote => Focus::DayNote,
            Focus::Period(focus) => match moved.get(focus.period).copied().flatten() {
                Some(period) => Focus::Period(FieldFocus { period, ..focus }),
                None => Focus::START,
            },
        };
    }

    fn show_problem(&mut self, problem: ConsistencyProblem) {
        self.notify(problem.message);
        self.inconsistent_periods = problem.periods;
    }

    /// Sorts every day and removes empty periods, then checks all days. For
    /// the earliest failing day, shows a dialog with `anyway` as the button
    /// that continues regardless; returns whether all days passed. Sorting
    /// alone isn't an unsaved change.
    fn check_all_days(&mut self, anyway: (&'static str, DialogAction)) -> bool {
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
                self.dialog = Some(Dialog::inconsistent_day(question, anyway));
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
        self.dialog = Some(Dialog::save_before_quitting());
        Outcome::Continue
    }

    /// Selects the day from a consistency-check dialog and shows its problem.
    fn go_to_inconsistent_day(&mut self) {
        if let Some(inconsistent) = self.inconsistent_day.take() {
            self.select_day(inconsistent.date);
            self.show_problem(inconsistent.problem);
        }
    }

    /// Ctrl+Enter: focuses From of the next period; on the day note, which
    /// comes after the last period, it does nothing.
    fn focus_next_period(&mut self) {
        if let Focus::Period(focus) = self.focus {
            self.focus_period_after(focus.period);
        }
    }

    /// Focuses From of the period after `current`. In the last period, adds
    /// a period after it and focuses its From, unless the last period is still
    /// open (without To, which includes an empty last period): then it only
    /// notifies. On the empty period of a day without periods it does nothing.
    fn focus_period_after(&mut self, current: usize) {
        if self.shows_empty_period() {
            return;
        }
        let next = current + 1;
        let period = if next < self.period_count() {
            next
        } else if self.shown_day().periods.last().is_some_and(Period::is_open) {
            self.notify("The last period is still open");
            return;
        } else {
            self.edit_selected_day(Day::add_period)
        };
        self.focus = Focus::Period(FieldFocus {
            period,
            field: Field::From,
        });
    }

    fn run_dialog_action(&mut self, action: DialogAction) -> Outcome {
        match action {
            DialogAction::DeletePeriod => self.delete_focused_period(),
            DialogAction::GoToDay => self.go_to_inconsistent_day(),
            DialogAction::SaveAnyway => return self.save(Overwrite::IfUnchanged),
            DialogAction::QuitAnyway => return self.quit(),
            DialogAction::Save => {
                self.quit_after_saving = true;
                return self.save(Overwrite::IfUnchanged);
            }
            DialogAction::DontSave => return Outcome::Quit,
            DialogAction::Overwrite => return self.save(Overwrite::Always),
            DialogAction::Cancel => self.quit_after_saving = false,
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
                self.notify("Saved");
            }
            Err(SaveError::ChangedOnDisk) => self.dialog = Some(Dialog::file_changed_on_disk()),
            Err(e @ SaveError::Io(_)) => {
                self.notify(e.to_string());
                self.quit_after_saving = false;
            }
        }
        Outcome::Continue
    }

    /// Deletes the focused period; focus moves to the period taking its place.
    /// After deleting the only period, the day shows an empty period, with
    /// focus on its From.
    fn delete_focused_period(&mut self) {
        if let Focus::Period(focus) = self.focus {
            let count = self.edit_selected_day(|day| {
                day.remove_period(focus.period);
                day.periods.len()
            });
            self.focus = if count == 0 {
                Focus::START
            } else {
                Focus::Period(FieldFocus {
                    period: focus.period.min(count - 1),
                    ..focus
                })
            };
        }
    }

    /// Selects another day, focusing From of its first period.
    fn select_day(&mut self, date: Date) {
        self.selected = date;
        self.focus = Focus::START;
        self.first_visible_period_row = 0;
    }

    /// The number of periods the right panel shows: at least one.
    fn period_count(&self) -> usize {
        self.shown_day().periods.len()
    }

    /// ↑/↓ and PgUp/PgDn: the closest field `periods` rows
    /// above/below, stopping at the first period and the day note.
    /// Implied-break rows aren't focusable, so they are skipped. Every period
    /// row has the same fields in the same columns, so the closest field is
    /// the same field, except that the part-of-day field stands for both From
    /// and To; leaving it toward a from–to row lands on From (see
    /// `fit_focus_to_period`). The day note is the row below the last period;
    /// above it, From is the closest field.
    fn move_focus_vertically(&mut self, direction: Direction, periods: usize) {
        let day_note_row = self.period_count();
        self.focus = match (direction, self.focus) {
            (Direction::Forward, Focus::DayNote) => Focus::DayNote,
            (Direction::Backward, Focus::DayNote) => Focus::Period(FieldFocus {
                period: day_note_row.saturating_sub(periods),
                field: Field::FIRST,
            }),
            (Direction::Backward, Focus::Period(focus)) => Focus::Period(FieldFocus {
                period: focus.period.saturating_sub(periods),
                ..focus
            }),
            (Direction::Forward, Focus::Period(focus))
                if focus.period + periods >= day_note_row =>
            {
                Focus::DayNote
            }
            (Direction::Forward, Focus::Period(focus)) => Focus::Period(FieldFocus {
                period: focus.period + periods,
                ..focus
            }),
        };
    }

    /// Home/End: `field` of the focused period, i.e. its first or last field
    /// (see `fit_focus_to_period` for a part of day). The day note is alone
    /// in its row.
    fn move_focus_to_field(&mut self, field: Field) {
        if let Focus::Period(focus) = self.focus {
            self.focus = Focus::Period(FieldFocus { field, ..focus });
        }
    }

    /// ←/→: the previous/next field of the focused period, stopping at its
    /// first and last field. The day note is alone in its row.
    fn move_focus_within_period(&mut self, direction: Direction) {
        if let Focus::Period(focus) = self.focus {
            let kind = self.extent_kind(focus.period);
            if let Some(field) = focus.field.neighbor(kind, direction) {
                self.focus = Focus::Period(FieldFocus { field, ..focus });
            }
        }
    }

    /// Tab/Shift-Tab: the next/previous field, continuing in the next/previous
    /// period and wrapping around within the day. The day note comes last:
    /// after the last field of the last period, and before the first period.
    fn move_focus_in_tab_order(&mut self, direction: Direction) {
        let last_period = self.period_count() - 1;
        self.focus = match (direction, self.focus) {
            (Direction::Forward, Focus::DayNote) => Focus::START,
            (Direction::Backward, Focus::DayNote) => Focus::Period(FieldFocus {
                period: last_period,
                field: Field::LAST,
            }),
            (direction, Focus::Period(focus)) => {
                let kind = self.extent_kind(focus.period);
                match (direction, focus.field.neighbor(kind, direction)) {
                    (_, Some(field)) => Focus::Period(FieldFocus { field, ..focus }),
                    (Direction::Forward, None) if focus.period == last_period => Focus::DayNote,
                    (Direction::Forward, None) => Focus::Period(FieldFocus {
                        period: focus.period + 1,
                        field: Field::FIRST,
                    }),
                    (Direction::Backward, None) if focus.period == 0 => Focus::DayNote,
                    (Direction::Backward, None) => Focus::Period(FieldFocus {
                        period: focus.period - 1,
                        field: Field::LAST,
                    }),
                }
            }
        };
    }
}

/// An input field's value in `period`, as it is shown; `None` for checkboxes
/// and empty values.
fn field_text(period: &Period, field: Field) -> Option<String> {
    match field {
        Field::From => period.from().map(TimeOfDay::format_hhmm),
        Field::To => period.to().map(TimeOfDay::format_hhmm),
        Field::Project => period.project.clone(),
        Field::Notes => period.notes.clone(),
        Field::PartOfDay
        | Field::Break
        | Field::Free
        | Field::Vacation
        | Field::Sick
        | Field::Holiday => None,
    }
}
