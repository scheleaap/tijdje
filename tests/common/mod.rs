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

pub fn row_containing<'a>(rows: &'a [String], text: &str) -> &'a str {
    rows.iter()
        .find(|r| r.contains(text))
        .unwrap_or_else(|| panic!("no row contains {text:?}"))
}

pub fn press(app: &mut App, code: KeyCode) -> tijdje::app::Outcome {
    app.handle_key(KeyEvent::new(code, KeyModifiers::NONE))
}

pub fn ctrl(app: &mut App, c: char) -> tijdje::app::Outcome {
    app.handle_key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL))
}
