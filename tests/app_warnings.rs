mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::app::Outcome;

#[test]
fn a_needless_split_gets_a_period_warning_and_a_break_shows_its_type() {
    let yaml = r#"
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: "12:00", project: acme, notes: null, type: work }
      - { from: "12:00", to: "12:30", project: acme, notes: null, type: work }
      - { from: "12:30", to: "13:00", project: null, notes: null, type: break }
"#;
    let mut app = app_with(yaml, date(2026, 11, 2));
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "0900 1200")).starts_with("│  "));
    assert!(right(row_containing(&screen, "1200 1230")).starts_with("│❗"));
    assert!(right(row_containing(&screen, "1230 1300")).starts_with("│☕"));
}

#[test]
fn an_implied_break_shows_a_break_indicator_and_lines_up_with_from() {
    // Given an implied break 1200–1230 between two work periods
    let yaml = r#"
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: "12:00", project: acme, notes: null, type: work }
      - { from: "12:30", to: "15:00", project: acme, notes: null, type: work }
"#;
    let mut app = app_with(yaml, date(2026, 11, 2));

    // When
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);

    // Then
    assert!(right(row_containing(&screen, "1200 1230")).starts_with("│☕"));
    let from_column = |text: &str| {
        let y = u16::try_from(row_index(&screen, text)).unwrap();
        column_of(&buffer, y, text)
    };
    assert_eq!(from_column("1200 1230"), from_column("0900 1200"));
    assert!(right(row_containing(&screen, "0900 1200")).starts_with("│  "));
    assert!(right(row_containing(&screen, "1230 1500")).starts_with("│  "));
}

#[test]
fn an_implied_break_next_to_a_consistency_failure_still_shows_a_break_indicator() {
    // Given an implied break 1200–1230 before an open period on a past day
    let yaml = r#"
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: "12:00", project: acme, notes: null, type: work }
      - { from: "12:30", to: null, project: acme, notes: null, type: work }
"#;
    let mut app = app_with(yaml, date(2026, 11, 3));
    ctrl_key(&mut app, KeyCode::Up);

    // When
    let screen = rows(&render(&mut app, 120, 24));

    // Then
    assert!(right(row_containing(&screen, "1200 1230")).starts_with("│☕"));
    assert!(right(row_containing(&screen, "1230     ")).starts_with("│⚠️"));
}

#[test]
fn a_consistency_failure_shows_right_away_while_filling_in_the_day() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "0845 1230")).starts_with("│⚠️"));
    assert!(right(row_containing(&screen, "1215 1300")).starts_with("│⚠️"));
    assert!(right(row_containing(&screen, "1300 1757")).starts_with("│  "));
}

#[test]
fn a_zero_minute_period_is_a_consistency_failure_not_a_warning() {
    // The last period's From, so no To changes and nothing chains.
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1757");
    press(&mut app, KeyCode::Enter);
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "1757 1757")).starts_with("│⚠️"));
    assert!(left(row_containing(&screen, "ma  2 nov")).contains("⚠️"));
}

#[test]
fn the_left_panel_never_shows_a_period_warning() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 40));
    assert!(screen.iter().all(|row| !left(row).contains('❗')));
    let thursday = left(row_containing(&screen, "do  5 nov"));
    assert!(!thursday.contains("⚠️"), "{thursday:?}");
}

const MONDAY_ONLY: &str = r#"
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: "16:12", project: acme, notes: null, type: work }
"#;

#[test]
fn an_empty_past_day_gets_a_day_warning() {
    let mut app = app_with(MONDAY_ONLY, date(2026, 11, 4));
    let screen = rows(&render(&mut app, 120, 24));
    assert!(left(row_containing(&screen, "di  3 nov")).contains("⚠️"));
    assert!(!left(row_containing(&screen, "ma  2 nov")).contains("⚠️"));
}

#[test]
fn saving_does_not_ask_about_an_empty_past_day() {
    let (mut app, fs) = app_with_fs(MONDAY_ONLY, date(2026, 11, 4));
    // Select Tue 3 Nov, and check and uncheck Break on its empty period.
    ctrl_key(&mut app, KeyCode::Up);
    assert_eq!(app.selected_day(), date(2026, 11, 3));
    alt_key(&mut app, KeyCode::Char('b'));
    alt_key(&mut app, KeyCode::Char('b'));
    assert!(app.has_unsaved_changes());
    let screen = rows(&render(&mut app, 120, 24));
    assert!(left(row_containing(&screen, "di  3 nov")).contains("⚠️"));

    assert_eq!(ctrl(&mut app, 's'), Outcome::Continue);
    assert!(!app.has_unsaved_changes());
    assert!(fs.get(TIMESHEET).unwrap().contains("2026-11-02"));
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Quit);
}
