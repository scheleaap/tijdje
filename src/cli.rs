//! The command line: `tijdje [PATH]`, `tijdje --help` and `tijdje --version`.

use std::{ffi::OsString, path::PathBuf};

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Invocation {
    /// Start the application on the given timesheet, or the default one.
    Run(Option<PathBuf>),
    Help,
    Version,
}

/// The usage, printed by `--help`.
pub fn usage() -> String {
    "Usage: tijdje [PATH]\n\
     \n\
     Records working time in the timesheet at PATH.\n\
     PATH defaults to ~/Documents/timesheet.yaml.\n\
     \n\
     Options:\n\
     \x20 --help     Print this help and exit\n\
     \x20 --version  Print the version and exit\n"
        .to_owned()
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
