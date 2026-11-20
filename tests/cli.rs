//! The command line: parsing the arguments, and the binary's output and exit codes.

use std::{ffi::OsString, path::PathBuf, process::Command};

use tijdje::cli::{self, Invocation};

fn parse(args: &[&str]) -> Result<Invocation, String> {
    cli::parse(args.iter().map(OsString::from))
}

#[test]
fn no_arguments_use_the_default_path() {
    assert_eq!(parse(&[]), Ok(Invocation::Run(None)));
}

#[test]
fn a_single_non_option_argument_is_the_timesheet_path() {
    assert_eq!(
        parse(&["work/timesheet.yaml"]),
        Ok(Invocation::Run(Some(PathBuf::from("work/timesheet.yaml"))))
    );
}

#[test]
fn help_and_version_are_recognized() {
    assert_eq!(parse(&["--help"]), Ok(Invocation::Help));
    assert_eq!(parse(&["--version"]), Ok(Invocation::Version));
}

#[test]
fn an_unknown_option_is_an_error_naming_it() {
    for option in ["--bogus", "-h", "-", "--"] {
        let err = parse(&[option]).unwrap_err();
        assert!(err.contains(&format!("'{option}'")), "{err}");
    }
}

#[test]
fn an_unknown_option_next_to_a_path_is_an_error() {
    assert!(parse(&["timesheet.yaml", "--bogus"]).is_err());
}

#[test]
fn more_than_one_path_is_an_error() {
    assert!(parse(&["a.yaml", "b.yaml"]).is_err());
}

#[test]
fn the_usage_names_the_default_path() {
    let usage = cli::usage();
    assert!(usage.contains("tijdje [PATH]"), "{usage}");
    assert!(usage.contains("~/Documents/timesheet.yaml"), "{usage}");
    assert!(usage.contains("--help"), "{usage}");
    assert!(usage.contains("--version"), "{usage}");
}

fn run(arg: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tijdje"))
        .arg(arg)
        .output()
        .unwrap()
}

#[test]
fn help_prints_the_usage_and_succeeds() {
    let output = run("--help");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), cli::usage());
}

#[test]
fn version_prints_the_version_and_succeeds() {
    let output = run("--version");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), cli::version());
}

#[test]
fn an_unknown_option_prints_an_error_and_fails() {
    let output = run("--bogus");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("'--bogus'"), "{stderr}");
}

#[test]
fn the_default_path_is_documents_timesheet_in_the_home_folder() {
    assert_eq!(
        cli::default_timesheet_path(std::path::Path::new("/home/x")),
        std::path::PathBuf::from("/home/x/Documents/timesheet.yaml")
    );
}
