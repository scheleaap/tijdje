mod common;

use common::{day, period};
use jiff::civil::{Date, date};
use tijdje::domain::{Period, PeriodType::Work, Timesheet};

fn with_project(project: &str) -> Period {
    Period {
        project: Some(project.into()),
        ..period(900, Some(1000), Work)
    }
}

/// A timesheet with one day per entry, each with periods on the given projects.
fn timesheet(days: Vec<(Date, Vec<&str>)>) -> Timesheet {
    Timesheet {
        days: days
            .into_iter()
            .map(|(date, projects)| (date, day(projects.into_iter().map(with_project).collect())))
            .collect(),
    }
}

#[test]
fn nothing_is_suggested_without_earlier_projects() {
    // Given
    let timesheet = Timesheet::default();

    // When
    let suggestion = timesheet.project_suggestion("o");

    // Then
    assert_eq!(suggestion, None);
}

#[test]
fn a_project_starting_with_the_typed_text_is_suggested() {
    // Given
    let timesheet = timesheet(vec![(date(2026, 11, 2), vec!["otto", "intern"])]);

    // When
    let suggestion = timesheet.project_suggestion("ot");

    // Then
    assert_eq!(suggestion, Some("otto"));
}

#[test]
fn nothing_is_suggested_for_empty_input() {
    // Given
    let timesheet = timesheet(vec![(date(2026, 11, 2), vec!["otto"])]);

    // When
    let suggestion = timesheet.project_suggestion("");

    // Then
    assert_eq!(suggestion, None);
}

#[test]
fn nothing_is_suggested_when_no_project_starts_with_the_typed_text() {
    // Given
    let timesheet = timesheet(vec![(date(2026, 11, 2), vec!["otto"])]);

    // When
    let suggestion = timesheet.project_suggestion("tt");

    // Then
    assert_eq!(suggestion, None);
}

#[test]
fn nothing_is_suggested_when_the_typed_text_is_already_complete() {
    // Given
    let timesheet = timesheet(vec![(date(2026, 11, 2), vec!["otto"])]);

    // When
    let suggestion = timesheet.project_suggestion("Otto");

    // Then
    assert_eq!(suggestion, None);
}

#[test]
fn case_is_ignored_and_the_suggestion_keeps_the_projects_spelling() {
    // Given
    let timesheet = timesheet(vec![(date(2026, 11, 2), vec!["Moco-Intern"])]);

    // When
    let suggestion = timesheet.project_suggestion("moco-i");

    // Then
    assert_eq!(suggestion, Some("Moco-Intern"));
}

#[test]
fn projects_differing_only_in_case_are_counted_separately() {
    // Given
    let timesheet = timesheet(vec![
        (date(2026, 11, 2), vec!["Acme", "Acme"]),
        (date(2026, 11, 3), vec!["ACME"]),
    ]);

    // When
    let suggestion = timesheet.project_suggestion("ac");

    // Then
    assert_eq!(suggestion, Some("Acme"));
}

#[test]
fn the_most_used_project_is_suggested() {
    // Given
    let timesheet = timesheet(vec![
        (date(2026, 11, 2), vec!["otto", "otto"]),
        (date(2026, 11, 3), vec!["ottobock"]),
    ]);

    // When
    let suggestion = timesheet.project_suggestion("ot");

    // Then
    assert_eq!(suggestion, Some("otto"));
}

#[test]
fn of_equally_used_projects_the_most_recently_used_is_suggested() {
    // Given
    let timesheet = timesheet(vec![
        (date(2026, 11, 2), vec!["ottobock"]),
        (date(2026, 11, 3), vec!["otto"]),
        (date(2026, 11, 4), vec!["ottobock"]),
        (date(2026, 11, 5), vec!["intern", "otto"]),
    ]);

    // When
    let suggestion = timesheet.project_suggestion("ot");

    // Then
    assert_eq!(suggestion, Some("otto"));
}
