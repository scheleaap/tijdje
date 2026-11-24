mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::app::{App, Field, Outcome};

/// Monday 2 Nov from the sample with the project of its first work period
/// (0845–1230) cleared.
fn monday_without_a_project() -> (App, InMemoryFileSystem) {
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    tab_to(&mut app, Field::Project);
    press(&mut app, KeyCode::Delete);
    (app, fs)
}

fn screen_rows(app: &mut App) -> Vec<String> {
    rows(&render(app, 120, 24))
}

fn screen_text(app: &mut App) -> String {
    screen_rows(app).join("\n")
}

/// The right-panel part of the first row whose right panel contains `text`.
fn right_row_containing(rows: &[String], text: &str) -> String {
    rows.iter()
        .map(|row| right(row))
        .find(|row| row.contains(text))
        .unwrap_or_else(|| panic!("no right-panel row contains {text:?}"))
}

#[test]
fn a_work_period_without_a_project_shows_a_consistency_failure_right_away() {
    // Given
    let (mut app, _fs) = monday_without_a_project();

    // When
    let shown = screen_rows(&mut app);

    // Then
    assert!(right_row_containing(&shown, "0845 1230").starts_with("│⚠️"));
    assert!(!right_row_containing(&shown, "1300 1757").starts_with("│⚠️"));
    assert!(left(row_containing(&shown, "ma  2 nov")).contains("⚠️"));
}

#[test]
fn an_open_work_period_today_without_a_project_shows_a_consistency_failure() {
    // Given today, 11 Nov, is empty
    let mut app = app_with(SAMPLE, date(2026, 11, 11));

    // When
    type_text(&mut app, "0900");
    press(&mut app, KeyCode::Enter);

    // Then
    let shown = screen_rows(&mut app);
    assert!(right_row_containing(&shown, "0900").starts_with("│⚠️"));
    assert!(left(row_containing(&shown, "wo 11 nov")).contains("⚠️"));
}

#[test]
fn leaving_a_changed_day_with_a_work_period_without_a_project_is_refused() {
    // Given
    let (mut app, _fs) = monday_without_a_project();

    // When
    ctrl_key(&mut app, KeyCode::Down);

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 2));
    assert!(screen_text(&mut app).contains("Work period 0845–1230 has no project"));
}

#[test]
fn ctrl_s_with_a_work_period_without_a_project_asks_first() {
    // Given
    let (mut app, fs) = monday_without_a_project();

    // When
    ctrl(&mut app, 's');

    // Then
    let text = screen_text(&mut app);
    assert!(
        text.contains("ma 2 nov: Work period 0845–1230 has no project"),
        "{text}"
    );
    assert!(text.contains(" Save anyway "), "{text}");
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some(SAMPLE));
}

#[test]
fn ctrl_q_with_a_work_period_without_a_project_asks_first() {
    // Given
    let (mut app, _fs) = monday_without_a_project();

    // When
    let outcome = ctrl(&mut app, 'q');

    // Then
    assert_eq!(outcome, Outcome::Continue);
    let text = screen_text(&mut app);
    assert!(
        text.contains("ma 2 nov: Work period 0845–1230 has no project"),
        "{text}"
    );
    assert!(text.contains(" Quit anyway "), "{text}");
}

#[test]
fn unchanged_older_days_without_a_project_can_be_browsed() {
    // Given Monday has a work period without a project, and today is Wednesday
    let yaml = one_day_yaml(
        "2026-11-02",
        &[r#"{ from: "09:00", to: "17:00", project: null, notes: null, type: work }"#],
    );
    let mut app = app_with(&yaml, date(2026, 11, 4));
    while app.selected_day() > date(2026, 11, 2) {
        ctrl_key(&mut app, KeyCode::Up);
    }

    // When
    ctrl_key(&mut app, KeyCode::Up);

    // Then
    assert_eq!(app.selected_day(), date(2026, 10, 30));
}

#[test]
fn a_typed_project_of_only_spaces_is_no_project() {
    // Given
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    tab_to(&mut app, Field::Project);

    // When
    type_text(&mut app, "   ");
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(period_at(&app, 0).project, None);
    assert!(right_row_containing(&screen_rows(&mut app), "0845 1230").starts_with("│⚠️"));
}

#[test]
fn a_pasted_project_of_only_spaces_and_line_breaks_is_no_project() {
    // Given
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    app.set_clipboard(Box::new(FakeClipboard::with("  \n  ")));
    tab_to(&mut app, Field::Project);

    // When
    ctrl(&mut app, 'v');

    // Then
    assert_eq!(period_at(&app, 0).project, None);
    assert!(!app.is_editing());
}
