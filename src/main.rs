use std::{env, path::PathBuf, process::ExitCode};

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use tijdje::{
    app::{App, Outcome},
    cli::{self, Invocation},
    clipboard::SystemClipboard,
    store::RealFileSystem,
    wide_symbols,
};

/// `~/Documents/timesheet.yaml`.
fn default_timesheet_path() -> PathBuf {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    home.join("Documents").join("timesheet.yaml")
}

fn main() -> ExitCode {
    let path = match cli::parse(env::args_os().skip(1)) {
        Ok(Invocation::Run(path)) => path.unwrap_or_else(default_timesheet_path),
        Ok(Invocation::Help) => {
            print!("{}", cli::usage());
            return ExitCode::SUCCESS;
        }
        Ok(Invocation::Version) => {
            print!("{}", cli::version());
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("tijdje: {e}\nTry 'tijdje --help' for more information.");
            return ExitCode::from(2);
        }
    };
    let today = jiff::Zoned::now().date();
    let mut app = match App::open(Box::new(RealFileSystem), path.clone(), today) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("tijdje: {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    app.set_clipboard(Box::new(SystemClipboard::new()));
    let mut terminal = ratatui::init();
    let result = (|| -> std::io::Result<()> {
        loop {
            let frame = terminal.draw(|frame| app.render(frame))?;
            wide_symbols::paint_backgrounds(frame.buffer, &mut std::io::stdout())?;
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
