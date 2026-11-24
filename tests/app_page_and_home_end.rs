mod common;

use common::*;
use jiff::civil::{Date, date};
use ratatui::crossterm::event::KeyCode;
use tijdje::app::{App, Field};

const DAY: Date = date(2026, 11, 12);

const FIRST_HALF_OF_VACATION: &str = r#"{ from: null, to: null, part_of_day: first_half, project: null, notes: null, type: vacation }"#;

/// Thu 12 Nov, today, with 12 hourly periods: indexes 0 to 11.
fn app_with_twelve_periods() -> App {
    app_with(&hourly_periods_yaml("2026-11-12", 12), DAY)
}

/// Thu 12 Nov, today, with a part-of-day period first, then 11 hourly
/// periods.
fn app_with_a_part_of_day_period_first() -> App {
    let periods: Vec<String> = std::iter::once(FIRST_HALF_OF_VACATION.to_owned())
        .chain((8..19).map(hourly_period))
        .collect();
    app_with(&one_day_yaml("2026-11-12", &periods), DAY)
}

/// Presses `code` `times` times.
fn press_times(app: &mut App, code: KeyCode, times: usize) {
    (0..times).for_each(|_| {
        press(app, code);
    });
}

// PgUp/PgDn

#[test]
fn page_down_moves_ten_periods_down_keeping_the_field() {
    // Given
    let mut app = app_with_twelve_periods();
    press(&mut app, KeyCode::Right);

    // When
    press(&mut app, KeyCode::PageDown);

    // Then
    assert_eq!(app.focus(), focus(10, Field::To));
}

#[test]
fn page_down_stops_at_the_last_period() {
    // Given: period 10, with only period 11 below it.
    let mut app = app_with_twelve_periods();
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::PageDown);

    // When
    press(&mut app, KeyCode::PageDown);

    // Then
    assert_eq!(app.focus(), focus(11, Field::To));
}

#[test]
fn page_down_does_nothing_on_the_last_period() {
    // Given
    let mut app = app_with_twelve_periods();
    press_times(&mut app, KeyCode::Down, 11);

    // When
    press(&mut app, KeyCode::PageDown);

    // Then: only ↓ goes on to the day note.
    assert_eq!(app.focus(), focus(11, Field::From));
}

#[test]
fn page_down_does_nothing_on_the_day_note() {
    // Given
    let mut app = app_with_twelve_periods();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::PageDown);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn page_down_from_the_period_ten_above_the_day_note_goes_to_the_last_period() {
    // Given: the day note is the row 10 below period 2.
    let mut app = app_with_twelve_periods();
    press_times(&mut app, KeyCode::Down, 2);

    // When
    press(&mut app, KeyCode::PageDown);

    // Then
    assert_eq!(app.focus(), focus(11, Field::From));
}

#[test]
fn page_up_moves_ten_periods_up_keeping_the_field() {
    // Given
    let mut app = app_with_twelve_periods();
    press_times(&mut app, KeyCode::Down, 11);
    tab_to(&mut app, Field::Project);

    // When
    press(&mut app, KeyCode::PageUp);

    // Then
    assert_eq!(app.focus(), focus(1, Field::Project));
}

#[test]
fn page_up_stops_at_the_first_period() {
    // Given: period 5, with fewer than 10 periods above it.
    let mut app = app_with_twelve_periods();
    press_times(&mut app, KeyCode::Down, 5);
    tab_to(&mut app, Field::Project);

    // When
    press(&mut app, KeyCode::PageUp);

    // Then
    assert_eq!(app.focus(), focus(0, Field::Project));
}

#[test]
fn page_up_does_nothing_on_the_first_period() {
    // Given
    let mut app = app_with_twelve_periods();
    tab_to(&mut app, Field::Project);

    // When
    press(&mut app, KeyCode::PageUp);

    // Then
    assert_eq!(app.focus(), focus(0, Field::Project));
}

#[test]
fn page_up_from_the_day_note_goes_to_from_of_the_period_ten_above_it() {
    // Given
    let mut app = app_with_twelve_periods();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::PageUp);

    // Then
    assert_eq!(app.focus(), focus(2, Field::From));
}

#[test]
fn page_up_from_the_day_note_goes_to_from_of_the_first_period_in_a_short_day() {
    // Given
    let mut app = app_on_monday();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::PageUp);

    // Then
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn page_up_from_the_day_note_goes_to_the_empty_period_of_a_day_without_periods() {
    // Given: Fri 13 Nov has no periods.
    let mut app = app_with(&hourly_periods_yaml("2026-11-12", 12), date(2026, 11, 13));
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::PageUp);

    // Then
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn page_keys_keep_the_matching_field_of_a_part_of_day_period() {
    // Given: To of period 10.
    let mut app = app_with_a_part_of_day_period_first();
    press_times(&mut app, KeyCode::Down, 10);
    press(&mut app, KeyCode::Right);
    let start = app.focus();

    // When
    press(&mut app, KeyCode::PageUp);
    let up = app.focus();
    press(&mut app, KeyCode::PageDown);

    // Then
    assert_eq!(start, focus(10, Field::To));
    assert_eq!(up, focus(0, Field::PartOfDay));
    assert_eq!(
        app.focus(),
        focus(10, Field::From),
        "leaving the part-of-day field lands on From, as with ↓"
    );
}

#[test]
fn page_keys_do_not_change_the_day() {
    // Given
    let mut app = app_with_twelve_periods();

    // When
    press(&mut app, KeyCode::PageDown);
    press(&mut app, KeyCode::PageUp);

    // Then
    assert_eq!(app.selected_day(), DAY);
}

#[test]
fn page_keys_are_ignored_while_editing() {
    // Given
    let mut app = app_with_twelve_periods();
    press_times(&mut app, KeyCode::Down, 5);
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "abc");

    // When
    press(&mut app, KeyCode::PageDown);
    press(&mut app, KeyCode::PageUp);
    type_text(&mut app, "d");
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(5, Field::Notes));
    assert_eq!(period_at(&app, 5).notes.as_deref(), Some("abcd"));
}

#[test]
fn the_right_panel_scrolls_to_the_period_page_down_focuses() {
    // Given
    let mut app = app_with_twelve_periods();

    // When
    press(&mut app, KeyCode::PageDown);

    // Then
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("1800"), "{screen}");
}

#[test]
fn the_right_panel_scrolls_to_the_period_page_up_focuses() {
    // Given: the day note has focus, reached with ↓, so the last periods
    // are shown and period 2 (1000–1100) is not.
    let mut app = app_with_twelve_periods();
    press_times(&mut app, KeyCode::Down, 11);
    render(&mut app, 120, 24);
    press(&mut app, KeyCode::Down);
    let before = rows(&render(&mut app, 120, 24)).join("\n");

    // When
    press(&mut app, KeyCode::PageUp);

    // Then
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert_eq!(app.focus(), focus(2, Field::From));
    assert!(!before.contains("1000"), "{before}");
    assert!(screen.contains("1000"), "{screen}");
}

// Home/End

#[test]
fn home_and_end_move_to_from_and_notes_of_the_focused_period() {
    // Given
    let mut app = app_with_twelve_periods();
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Project);

    // When
    press(&mut app, KeyCode::End);
    let after_end = app.focus();
    press(&mut app, KeyCode::Home);

    // Then
    assert_eq!(after_end, focus(1, Field::Notes));
    assert_eq!(app.focus(), focus(1, Field::From));
}

#[test]
fn home_moves_to_the_part_of_day_field_of_a_part_of_day_period() {
    // Given
    let mut app = app_with_a_part_of_day_period_first();
    press(&mut app, KeyCode::End);

    // When
    press(&mut app, KeyCode::Home);

    // Then
    assert_eq!(app.focus(), focus(0, Field::PartOfDay));
}

#[test]
fn end_moves_to_notes_of_a_part_of_day_period() {
    // Given
    let mut app = app_with_a_part_of_day_period_first();

    // When
    press(&mut app, KeyCode::End);

    // Then
    assert_eq!(app.focus(), focus(0, Field::Notes));
}

#[test]
fn home_and_end_do_nothing_on_the_day_note() {
    // Given
    let mut app = app_with_twelve_periods();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Home);
    let after_home = app.focus();
    press(&mut app, KeyCode::End);

    // Then
    assert_eq!(after_home, DAY_NOTE);
    assert_eq!(app.focus(), DAY_NOTE);
}

// Help

#[test]
fn the_help_window_lists_page_up_page_down_home_and_end() {
    // Given
    let mut app = app_with_twelve_periods();

    // When
    press(&mut app, KeyCode::F(1));

    // Then: on a screen on which the whole Help window fits
    let screen = rows(&render(&mut app, 120, 60));
    assert!(
        row_containing(&screen, " PgUp/PgDn ").contains("10 periods up/down"),
        "{screen:#?}"
    );
    assert!(
        row_containing(&screen, " Home/End ").contains("First/last field of the period"),
        "{screen:#?}"
    );
}
