#![allow(dead_code)]

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    io,
    path::{Path, PathBuf},
    rc::Rc,
};

use jiff::civil::Date;
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use tijdje::{
    app::{App, Field, FieldFocus, Focus},
    domain::{Day, Minutes, Period, PeriodType, TimeOfDay},
    store::FileSystem,
};

pub const SAMPLE: &str = include_str!("../../sample/timesheet.yaml");

/// Width of the left panel, in cells.
pub const LEFT_PANEL_WIDTH: usize = 46;

/// The part of a screen row that belongs to the left panel.
pub fn left(row: &str) -> String {
    row.chars().take(LEFT_PANEL_WIDTH).collect()
}

/// The part of a screen row that belongs to the right panel.
pub fn right(row: &str) -> String {
    row.chars().skip(LEFT_PANEL_WIDTH).collect()
}

/// The right panel's left padding at a screen width of 120: the first cell
/// inside its frame, before the day note and the bottom line.
pub const LEFT_PADDING_X: u16 = 47;
/// The right panel's right padding at a screen width of 120: the last cell
/// inside its frame, after the day note and the Notes field.
pub const RIGHT_PADDING_X: u16 = 118;

/// A right-panel row without its frame and the spaces next to it.
pub fn inside_frame(row: &str) -> &str {
    row.trim_matches(['│', ' '])
}

/// A day on `date` with `count` consecutive hourly periods from 08:00 on,
/// as timesheet YAML.
pub fn hourly_periods_yaml(date: &str, count: u16) -> String {
    let periods: String = (0..count)
        .map(|i| {
            format!(
                "      - {{ from: \"{:02}:00\", to: \"{:02}:00\", project: null, notes: null, type: work }}\n",
                8 + i,
                9 + i
            )
        })
        .collect();
    format!(
        "days:\n  \"{date}\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n{periods}"
    )
}

/// An in-memory file system; clones share the same files.
#[derive(Default, Clone)]
pub struct InMemoryFileSystem {
    files: Rc<RefCell<HashMap<PathBuf, String>>>,
}

impl InMemoryFileSystem {
    pub fn with(path: &str, content: &str) -> Self {
        let fs = Self::default();
        fs.set(path, content);
        fs
    }

    pub fn get(&self, path: &str) -> Option<String> {
        self.files.borrow().get(Path::new(path)).cloned()
    }

    pub fn set(&self, path: &str, content: &str) {
        self.files
            .borrow_mut()
            .insert(PathBuf::from(path), content.to_owned());
    }
}

impl FileSystem for InMemoryFileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.files
            .borrow()
            .get(path)
            .cloned()
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        self.files
            .borrow_mut()
            .insert(path.to_owned(), contents.to_owned());
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        let contents = self.read_to_string(from)?;
        self.write(to, &contents)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        let contents = self
            .files
            .borrow_mut()
            .remove(from)
            .ok_or(io::ErrorKind::NotFound)?;
        self.write(to, &contents)
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        self.files.borrow_mut().remove(path);
        Ok(())
    }

    fn resolve(&self, path: &Path) -> PathBuf {
        path.to_owned()
    }
}

pub const TIMESHEET: &str = "timesheet.yaml";

pub fn app_with(content: &str, today: Date) -> App {
    app_with_fs(content, today).0
}

/// An app on an in-memory timesheet file, and that file system.
pub fn app_with_fs(content: &str, today: Date) -> (App, InMemoryFileSystem) {
    app_with_clock(content, &FakeClock::new(today))
}

/// An app on an in-memory timesheet file whose "today" comes from `clock`,
/// and that file system.
pub fn app_with_clock(content: &str, clock: &FakeClock) -> (App, InMemoryFileSystem) {
    let fs = InMemoryFileSystem::with(TIMESHEET, content);
    let app = App::open(
        Box::new(fs.clone()),
        PathBuf::from(TIMESHEET),
        Box::new(clock.clone()),
    )
    .expect("valid timesheet");
    (app, fs)
}

/// A clock whose date the test sets; clones share the same date.
#[derive(Clone)]
pub struct FakeClock(Rc<Cell<Date>>);

impl FakeClock {
    pub fn new(today: Date) -> Self {
        Self(Rc::new(Cell::new(today)))
    }

    pub fn set(&self, today: Date) {
        self.0.set(today);
    }
}

impl tijdje::clock::Clock for FakeClock {
    fn today(&self) -> Date {
        self.0.get()
    }
}

pub fn render(app: &mut App, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    terminal.backend().buffer().clone()
}

/// The screen as one string per row.
pub fn rows(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect()
        })
        .collect()
}

/// Where `text` starts on row `y`, as a cell column.
pub fn column_of(buffer: &Buffer, y: u16, text: &str) -> Option<u16> {
    let chars: Vec<String> = text.chars().map(String::from).collect();
    (0..buffer.area.width).find(|&x| {
        chars.iter().enumerate().all(|(i, c)| {
            u16::try_from(i)
                .ok()
                .and_then(|i| x.checked_add(i))
                .filter(|&x| x < buffer.area.width)
                .is_some_and(|x| buffer[(x, y)].symbol() == c)
        })
    })
}

/// The first row containing `text`, skipping the top border row with the panel titles.
pub fn row_containing<'a>(rows: &'a [String], text: &str) -> &'a str {
    &rows[row_index(rows, text)]
}

pub fn press(app: &mut App, code: KeyCode) -> tijdje::app::Outcome {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
}

pub fn ctrl(app: &mut App, c: char) -> tijdje::app::Outcome {
    app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
}

/// A period from `from` to `to`, both written as HHMM numbers (e.g. `1235`).
pub fn period(from: u16, to: Option<u16>, period_type: PeriodType) -> Period {
    let time = |hhmm: u16| TimeOfDay::new(hhmm / 100, hhmm % 100);
    Period {
        from: time(from),
        to: to.and_then(time),
        project: None,
        notes: None,
        period_type,
    }
}

pub fn alt_key(app: &mut App, code: KeyCode) -> tijdje::app::Outcome {
    app.handle_key(KeyEvent::new(code, KeyModifiers::ALT))
}

pub fn ctrl_key(app: &mut App, code: KeyCode) -> tijdje::app::Outcome {
    app.handle_key(KeyEvent::new(code, KeyModifiers::CONTROL))
}

/// The index of the first row containing `text`, skipping the top border row.
pub fn row_index(rows: &[String], text: &str) -> usize {
    rows.iter()
        .skip(1)
        .position(|r| r.contains(text))
        .map(|i| i + 1)
        .unwrap_or_else(|| panic!("no row contains {text:?}"))
}

/// A day with the default regular expected working time of 7h12m.
pub fn day(periods: Vec<Period>) -> Day {
    Day {
        regular_expected_working_time: Some(Minutes(432)),
        periods,
        ..Day::default()
    }
}

/// Focuses the last period of the selected day, starting from the first
/// period, and adds a period after it with Ctrl+Enter.
pub fn add_period(app: &mut App) {
    let periods = app.timesheet().day(app.selected_day()).periods.len();
    for _ in 1..periods {
        press(app, KeyCode::Down);
    }
    ctrl_key(app, KeyCode::Enter);
}

/// Types `text` key by key.
pub fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        press(app, KeyCode::Char(c));
    }
}

/// The day on which the sample is consistent: its open period (10 Nov) is today.
pub const SAMPLE_TODAY: Date = jiff::civil::date(2026, 11, 10);

/// The sample on an in-memory file, with "today" set so the sample passes the
/// consistency check, and `day` selected.
pub fn sample_on(day: Date) -> (App, InMemoryFileSystem) {
    let (mut app, fs) = app_with_fs(SAMPLE, SAMPLE_TODAY);
    while app.selected_day() < day {
        ctrl_key(&mut app, KeyCode::Down);
    }
    while app.selected_day() > day {
        ctrl_key(&mut app, KeyCode::Up);
    }
    (app, fs)
}

/// An in-memory clipboard; clones share the same contents.
#[derive(Default, Clone)]
pub struct FakeClipboard(pub Rc<RefCell<Option<String>>>);

impl FakeClipboard {
    pub fn with(text: &str) -> Self {
        Self(Rc::new(RefCell::new(Some(text.to_owned()))))
    }

    pub fn text(&self) -> Option<String> {
        self.0.borrow().clone()
    }
}

impl tijdje::clipboard::Clipboard for FakeClipboard {
    fn get_text(&mut self) -> Option<String> {
        self.text()
    }

    fn set_text(&mut self, text: &str) -> bool {
        *self.0.borrow_mut() = Some(text.to_owned());
        true
    }
}

/// Mon 2 Nov from the sample: 0845–1230 work, 1230–1300 break, 1300–1757 work.
pub fn app_on_monday() -> App {
    app_with(SAMPLE, jiff::civil::date(2026, 11, 2))
}

/// The period at `index` of the selected day.
pub fn period_at(app: &App, index: usize) -> &Period {
    &app.timesheet().days[&app.selected_day()].periods[index]
}

/// Moves focus with Tab until `field` of some period has focus.
pub fn tab_to(app: &mut App, field: Field) {
    while app.focus().field() != Some(field) {
        press(app, KeyCode::Tab);
    }
}

/// Focus on `field` of the period at index `period`.
pub fn focus(period: usize, field: Field) -> Focus {
    Focus::Period(FieldFocus { period, field })
}

/// Focus on the day note.
pub const DAY_NOTE: Focus = Focus::DayNote;

/// Moves focus from From of the first period to the day note with Shift-Tab.
pub fn focus_day_note(app: &mut App) {
    press(app, KeyCode::BackTab);
    assert_eq!(
        app.focus(),
        DAY_NOTE,
        "focus_day_note starts on From of the first period"
    );
}
