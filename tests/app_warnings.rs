mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::app::{App, Outcome};

fn right(row: &str) -> String {
    row.chars().skip(46).collect()
}

fn left(row: &str) -> String {
    row.chars().take(46).collect()
}

/// Mon 2 Nov on the sample, with focus on the first period's To.
fn monday_on_the_first_to() -> App {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Tab);
    app
}

#[test]
fn a_needless_split_gets_a_period_warning_and_a_break_its_type() {
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
fn a_consistency_failure_shows_right_away_while_filling_in_the_day() {
    let mut app = monday_on_the_first_to();
    type_text(&mut app, "1245");
    press(&mut app, KeyCode::Enter);
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "0845 1245")).starts_with("│⚠️"));
    assert!(right(row_containing(&screen, "1230 1300")).starts_with("│⚠️"));
    assert!(right(row_containing(&screen, "1300 1757")).starts_with("│  "));
}

#[test]
fn a_zero_minute_period_is_a_consistency_failure_not_a_warning() {
    let mut app = monday_on_the_first_to();
    type_text(&mut app, "0845");
    press(&mut app, KeyCode::Enter);
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "0845 0845")).starts_with("│⚠️"));
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
    // Select Tue 3 Nov and add an empty period to it.
    press(&mut app, KeyCode::Up);
    assert_eq!(app.selected_day(), date(2026, 11, 3));
    alt_key(&mut app, KeyCode::Right);
    alt_key(&mut app, KeyCode::Enter);
    assert!(app.has_unsaved_changes());
    let screen = rows(&render(&mut app, 120, 24));
    assert!(left(row_containing(&screen, "di  3 nov")).contains("⚠️"));

    assert_eq!(ctrl(&mut app, 's'), Outcome::Continue);
    assert!(!app.has_unsaved_changes());
    assert!(fs.get(TIMESHEET).unwrap().contains("2026-11-02"));
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Quit);
}
