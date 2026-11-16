#![allow(dead_code)]

use std::{
    cell::RefCell,
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
    app::App,
    domain::{Day, Minutes, Period, PeriodType, TimeOfDay},
    store::FileSystem,
};

pub const SAMPLE: &str = include_str!("../../sample/timesheet.yaml");

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
    let fs = InMemoryFileSystem::with(TIMESHEET, content);
    let app =
        App::open(Box::new(fs.clone()), PathBuf::from(TIMESHEET), today).expect("valid timesheet");
    (app, fs)
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
    }
}

/// Types `text` key by key.
pub fn type_text(app: &mut App, text: &str) {
    for c in text.chars() {
        press(app, KeyCode::Char(c));
    }
}
