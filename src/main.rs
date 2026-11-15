use std::{env, path::PathBuf, process::ExitCode};

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use tijdje::{
    app::{App, Outcome},
    store::{self, RealFileSystem},
};

fn timesheet_path() -> PathBuf {
    env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(|| {
        let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
        home.join("Documents").join("timesheet.yaml")
    })
}

fn main() -> ExitCode {
    let path = timesheet_path();
    let timesheet = match store::load(&RealFileSystem, &path) {
        Ok(timesheet) => timesheet,
        Err(e) => {
            eprintln!("tijdje: {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    let mut app = App::new(timesheet, jiff::Zoned::now().date());
    let mut terminal = ratatui::init();
    let result = (|| -> std::io::Result<()> {
        loop {
            terminal.draw(|frame| app.render(frame))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && app.handle_key(key) == Outcome::Quit
            {
                return Ok(());
            }
        }
    })();
    ratatui::restore();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("tijdje: {e}");
            ExitCode::FAILURE
        }
    }
}
