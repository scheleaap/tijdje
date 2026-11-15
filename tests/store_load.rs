mod common;

use std::path::Path;

use common::InMemoryFileSystem;
use jiff::civil::date;
use tijdje::{
    domain::{Minutes, PeriodType, TimeOfDay},
    store::{self, LoadError},
};

fn load(content: &str) -> Result<tijdje::domain::Timesheet, LoadError> {
    store::load(
        &InMemoryFileSystem::with("t.yaml", content),
        Path::new("t.yaml"),
    )
}

#[test]
fn reads_the_documented_schema() {
    let timesheet = load(
        r#"
days:
  "2026-11-12":
    regular_expected_working_time_in_minutes: 432
    periods:
      - from: "12:35"
        to: "14:30"
        project: "otto"
        notes: foo! bar?
        type: work
      - from: "14:30"
        to: null
        project: ""
        notes: null
        type: break
"#,
    )
    .unwrap();
    let day = &timesheet.days[&date(2026, 11, 12)];
    assert_eq!(day.regular_expected_working_time, Some(Minutes(432)));
    assert_eq!(day.periods[0].from, TimeOfDay::new(12, 35));
    assert_eq!(day.periods[0].to, TimeOfDay::new(14, 30));
    assert_eq!(day.periods[0].project.as_deref(), Some("otto"));
    assert_eq!(day.periods[0].notes.as_deref(), Some("foo! bar?"));
    assert_eq!(day.periods[1].to, None);
    assert_eq!(
        day.periods[1].project, None,
        "empty string is read as empty"
    );
    assert_eq!(day.periods[1].period_type, PeriodType::Break);
}

#[test]
fn missing_file_is_an_empty_timesheet() {
    let timesheet = store::load(&InMemoryFileSystem::default(), Path::new("nope.yaml")).unwrap();
    assert!(timesheet.days.is_empty());
}

#[test]
fn unparsable_yaml_is_an_error() {
    assert!(matches!(load("days: [unclosed"), Err(LoadError::Parse(_))));
}

#[test]
fn invalid_time_is_an_error_naming_the_day() {
    let err = load(
        r#"
days:
  "2026-11-12":
    periods:
      - { from: "1235", to: null, project: null, notes: null, type: work }
"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("2026-11-12"), "{err}");
}

#[test]
fn sample_file_loads() {
    load(common::SAMPLE).unwrap();
}
