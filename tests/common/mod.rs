#![allow(dead_code)]

use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
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
    store::{self, FileSystem},
};

pub const SAMPLE: &str = include_str!("../../sample/timesheet.yaml");

#[derive(Default)]
pub struct InMemoryFileSystem {
    pub files: HashMap<PathBuf, String>,
}

impl InMemoryFileSystem {
    pub fn with(path: &str, content: &str) -> Self {
        Self {
            files: HashMap::from([(PathBuf::from(path), content.to_owned())]),
        }
    }
}

impl FileSystem for InMemoryFileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| io::ErrorKind::NotFound.into())
    }
}

pub fn app_with(content: &str, today: Date) -> App {
    let fs = InMemoryFileSystem::with("timesheet.yaml", content);
    App::new(
        store::load(&fs, Path::new("timesheet.yaml")).expect("valid timesheet"),
        today,
    )
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
