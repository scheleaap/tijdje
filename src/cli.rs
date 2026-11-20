//! The command line: `tijdje [PATH]`, `tijdje --help` and `tijdje --version`.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// Where the timesheet is by default, relative to the home folder.
const DEFAULT_PATH_IN_HOME: [&str; 2] = ["Documents", "timesheet.yaml"];

/// The default timesheet path in `home`: `<home>/Documents/timesheet.yaml`.
pub fn default_timesheet_path(home: &Path) -> PathBuf {
    DEFAULT_PATH_IN_HOME
        .iter()
        .fold(home.to_owned(), |path, part| path.join(part))
}

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    /// Start the application on the given timesheet, or the default one.
    Run(Option<PathBuf>),
    /// Print the usage and exit.
    Help,
    /// Print the version and exit.
    Version,
}

/// The usage, printed by `--help`.
pub fn usage() -> String {
    format!(
        "Usage: tijdje [PATH]\n\
         \n\
         Records working time in the timesheet at PATH.\n\
         PATH defaults to ~/{}.\n\
         \n\
         Options:\n\
         \x20 --help     Print this help and exit\n\
         \x20 --version  Print the version and exit\n",
        DEFAULT_PATH_IN_HOME.join("/")
    )
}

/// The version, printed by `--version`.
pub fn version() -> String {
    format!("tijdje {}\n", env!("CARGO_PKG_VERSION"))
}

/// Parses the arguments after the program name. Every argument that starts
/// with `-` is an option; only `--help` and `--version` are known. At most
/// one other argument is allowed: the timesheet path.
pub fn parse(args: impl IntoIterator<Item = OsString>) -> Result<Invocation, String> {
    let mut path = None;
    for arg in args {
        if arg == "--help" {
            return Ok(Invocation::Help);
        }
        if arg == "--version" {
            return Ok(Invocation::Version);
        }
        if arg.to_string_lossy().starts_with('-') {
            return Err(format!("unknown option '{}'", arg.to_string_lossy()));
        }
        if path.is_some() {
            return Err(format!(
                "unexpected argument '{}': only one PATH is allowed",
                arg.to_string_lossy()
            ));
        }
        path = Some(PathBuf::from(arg));
    }
    Ok(Invocation::Run(path))
}
