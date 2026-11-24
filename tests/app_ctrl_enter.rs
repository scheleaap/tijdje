//! Ctrl+Enter moves down like ↓ (adding a period in the last period) and
//! Ctrl+Shift+Enter moves up like ↑.

mod common;

use common::*;
use jiff::civil::{Date, date};
use ratatui::crossterm::event::KeyCode;
use tijdje::{app::Field, domain::TimeOfDay};

/// Mon 2 Nov from the sample.
const MONDAY: Date = date(2026, 11, 2);

/// Fri 6 Nov from the sample has one period, 0900–1612.
const FRIDAY: Date = date(2026, 11, 6);

/// A 1300–1400 work period, then a vacation 1st half, on Thu 12 Nov.
fn app_with_a_part_of_day_period_last() -> tijdje::app::App {
    let periods = [
        r#"{ from: "13:00", to: "14:00", project: acme, notes: null, type: work }"#,
        r#"{ from: null, to: null, part_of_day: first_half, project: null, notes: null, type: vacation }"#,
    ];
    app_with(&one_day_yaml("2026-11-12", &periods), date(2026, 11, 12))
}

fn period_count(app: &tijdje::app::App, day: Date) -> usize {
    app.timesheet().day(day).periods.len()
}

// Ctrl+Enter

#[test]
fn ctrl_enter_moves_down_onto_the_part_of_day_field_like_down() {
    // Given
    let mut app = app_with_a_part_of_day_period_last();
    press(&mut app, KeyCode::Right);

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(1, Field::PartOfDay));
    assert_eq!(period_count(&app, date(2026, 11, 12)), 2);
}

// Ctrl+Shift+Enter

#[test]
fn ctrl_shift_enter_moves_up_keeping_the_field() {
    // Given: the last period, where Ctrl+Enter would add one.
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Notes);

    // When
    ctrl_shift_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(1, Field::Notes));
    assert_eq!(period_count(&app, MONDAY), 3);
}

#[test]
fn ctrl_shift_enter_moves_up_from_the_part_of_day_field_to_from_like_up() {
    // Given
    let mut app = app_with_a_part_of_day_period_last();
    press(&mut app, KeyCode::Down);

    // When
    ctrl_shift_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn ctrl_shift_enter_does_nothing_on_the_first_period() {
    // Given: the only period is also the last, where Ctrl+Enter would add one.
    let mut app = app_with(SAMPLE, FRIDAY);
    tab_to(&mut app, Field::Project);

    // When
    ctrl_shift_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(0, Field::Project));
    assert_eq!(period_count(&app, FRIDAY), 1);
    assert!(!app.is_editing());
    assert!(!app.has_unsaved_changes());
}

#[test]
fn ctrl_shift_enter_from_the_day_note_goes_to_from_of_the_last_period() {
    // Given
    let mut app = app_on_monday();
    focus_day_note(&mut app);

    // When
    ctrl_shift_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(2, Field::From));
}

#[test]
fn ctrl_shift_enter_while_editing_accepts_and_moves_up_keeping_the_field() {
    // Given
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "lunch");

    // When
    ctrl_shift_key(&mut app, KeyCode::Enter);

    // Then
    assert!(!app.is_editing());
    assert_eq!(period_at(&app, 1).notes.as_deref(), Some("lunch"));
    assert_eq!(app.focus(), focus(0, Field::Notes));
}

#[test]
fn ctrl_shift_enter_with_an_invalid_time_stays_in_the_field() {
    // Given
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "99");

    // When
    ctrl_shift_key(&mut app, KeyCode::Enter);

    // Then
    assert!(app.is_editing());
    assert_eq!(app.focus(), focus(1, Field::From));
    assert_eq!(period_at(&app, 1).from(), TimeOfDay::new(12, 30));
}

#[test]
fn ctrl_shift_enter_while_editing_the_first_period_accepts_and_stays() {
    // Given
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "standup");

    // When
    ctrl_shift_key(&mut app, KeyCode::Enter);

    // Then
    assert!(!app.is_editing());
    assert_eq!(period_at(&app, 0).notes.as_deref(), Some("standup"));
    assert_eq!(app.focus(), focus(0, Field::Notes));
}

// Other Enter combinations

#[test]
fn shift_enter_does_nothing() {
    // Given
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Project);

    // When
    shift_key(&mut app, KeyCode::Enter);

    // Then
    assert!(!app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::Project));
}

#[test]
fn shift_enter_does_not_toggle_a_checkbox() {
    // Given
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Holiday);

    // When
    shift_key(&mut app, KeyCode::Enter);

    // Then
    assert!(!app.has_unsaved_changes());
}

#[test]
fn shift_enter_does_nothing_while_editing() {
    // Given
    let mut app = app_on_monday();
    type_text(&mut app, "0815");

    // When
    shift_key(&mut app, KeyCode::Enter);

    // Then
    assert!(app.is_editing());
    assert_eq!(period_at(&app, 0).from(), TimeOfDay::new(8, 45));
}

#[test]
fn ctrl_alt_enter_does_nothing() {
    // Given: the last period, where Ctrl+Enter would add one.
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);

    // When
    ctrl_alt_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(2, Field::From));
    assert_eq!(period_count(&app, MONDAY), 3);
    assert!(!app.has_unsaved_changes());
}

#[test]
fn ctrl_alt_enter_does_nothing_while_editing() {
    // Given
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1215");

    // When
    ctrl_alt_key(&mut app, KeyCode::Enter);

    // Then
    assert!(app.is_editing());
    assert_eq!(app.focus(), focus(1, Field::From));
    assert_eq!(period_at(&app, 1).from(), TimeOfDay::new(12, 30));
}
