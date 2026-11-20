use std::{env, path::PathBuf, process::ExitCode, time::Duration};

use ratatui::crossterm::event::{self, Event, KeyEventKind};
use tijdje::{
    app::{App, Outcome},
    cli::{self, Invocation},
    clipboard::SystemClipboard,
    clock::SystemClock,
    store::RealFileSystem,
    terminal, wide_symbols,
};

/// How long the event loop waits for an event before checking the clock again.
const CLOCK_CHECK_INTERVAL: Duration = Duration::from_secs(60);

fn default_timesheet_path() -> PathBuf {
    let home = env::var_os("HOME").map(PathBuf::from).unwrap_or_default();
    cli::default_timesheet_path(&home)
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
    if let Err(e) = terminal::check_keyboard_protocol() {
        eprintln!("tijdje: {e}");
        return ExitCode::FAILURE;
    }
    let mut app = match App::open(
        Box::new(RealFileSystem),
        path.clone(),
        Box::new(SystemClock),
    ) {
        Ok(app) => app,
        Err(e) => {
            eprintln!("tijdje: {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
    };
    app.set_clipboard(Box::new(SystemClipboard::new()));
    let mut terminal = match terminal::init() {
        Ok(terminal) => terminal,
        Err(e) => {
            eprintln!("tijdje: {e}");
            return ExitCode::FAILURE;
        }
    };
    let result = (|| -> std::io::Result<()> {
        loop {
            app.refresh_today();
            wide_symbols::draw(&mut terminal, &mut std::io::stdout(), |frame| {
                app.render(frame);
            })?;
            if event::poll(CLOCK_CHECK_INTERVAL)?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && app.handle_key(key) == Outcome::Quit
            {
                return Ok(());
            }
        }
    })();
    terminal::restore();
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("tijdje: {e}");
            ExitCode::FAILURE
        }
    }
}
