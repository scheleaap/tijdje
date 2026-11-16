mod common;

use common::*;
use jiff::civil::date;
use ratatui::{crossterm::event::KeyCode, style::Color};
use tijdje::{
    app::{App, Outcome, Panel},
    domain::TimeOfDay,
};

/// Monday 2 Nov with its first period changed to 0845–1245, which overlaps
/// the 1230–1300 break.
fn monday_with_an_overlap() -> (App, InMemoryFileSystem) {
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Tab);
    type_text(&mut app, "1245");
    press(&mut app, KeyCode::Enter);
    (app, fs)
}

/// The sample with today = 13 Nov: its open period on 10 Nov is then on a past day.
fn sample_with_a_past_open_period() -> App {
    app_with(SAMPLE, date(2026, 11, 13))
}

fn screen(app: &mut App) -> String {
    rows(&render(app, 120, 24)).join("\n")
}

#[test]
fn leaving_a_changed_inconsistent_day_is_refused_with_a_message() {
    let (mut app, _fs) = monday_with_an_overlap();
    alt_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 2));
    assert!(screen(&mut app).contains("0845–1245 overlaps 1230–1300"));
}

#[test]
fn the_offending_periods_are_highlighted_until_the_next_key() {
    let (mut app, _fs) = monday_with_an_overlap();
    alt_key(&mut app, KeyCode::Down);
    let buffer = render(&mut app, 120, 24);
    let rows = rows(&buffer);
    for text in ["0845 1245", "1230 1300"] {
        let y = row_index(&rows, text);
        assert_eq!(
            buffer[(60, u16::try_from(y).unwrap())].bg,
            Color::Red,
            "{text}"
        );
    }
    let y = row_index(&rows, "1300 1757");
    assert_ne!(buffer[(60, u16::try_from(y).unwrap())].bg, Color::Red);

    press(&mut app, KeyCode::Up);
    let text = screen(&mut app);
    assert!(!text.contains("overlaps"));
    let buffer = render(&mut app, 120, 24);
    let y = row_index(&rows, "0845 1245");
    assert_ne!(buffer[(60, u16::try_from(y).unwrap())].bg, Color::Red);
}

#[test]
fn leaving_from_the_left_panel_is_checked_too() {
    let (mut app, _fs) = monday_with_an_overlap();
    alt_key(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 2));
    press(&mut app, KeyCode::PageDown);
    assert_eq!(app.selected_day(), date(2026, 11, 2));
}

#[test]
fn unchanged_inconsistent_days_can_be_browsed() {
    let mut app = sample_with_a_past_open_period();
    while app.selected_day() > date(2026, 11, 10) {
        press(&mut app, KeyCode::Up);
    }
    press(&mut app, KeyCode::Up);
    assert_eq!(app.selected_day(), date(2026, 11, 9));
}

#[test]
fn a_fixed_day_can_be_left() {
    let (mut app, _fs) = monday_with_an_overlap();
    alt_key(&mut app, KeyCode::Down);
    type_text(&mut app, "1230");
    press(&mut app, KeyCode::Enter);
    alt_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 3));
}

#[test]
fn leaving_a_day_sorts_its_periods() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    alt_key(&mut app, KeyCode::Enter);
    type_text(&mut app, "0700");
    press(&mut app, KeyCode::Tab);
    type_text(&mut app, "0800");
    press(&mut app, KeyCode::Enter);
    alt_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 3));
    let monday = &app.timesheet().days[&date(2026, 11, 2)];
    assert_eq!(monday.periods[0].from, TimeOfDay::new(7, 0));
    assert_eq!(monday.periods[1].from, TimeOfDay::new(8, 45));
}

#[test]
fn leaving_a_day_removes_empty_periods() {
    let (mut app, _fs) = sample_on(date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Right);
    alt_key(&mut app, KeyCode::Enter);
    alt_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 12));
    assert!(app.timesheet().day(date(2026, 11, 11)).periods.is_empty());
}

#[test]
fn an_inconsistent_day_gets_a_day_warning() {
    let (mut app, _fs) = monday_with_an_overlap();
    let rows = rows(&render(&mut app, 120, 24));
    assert!(row_containing(&rows, "ma  2 nov").contains("⚠️"));
}

#[test]
fn ctrl_s_with_an_inconsistent_day_asks_what_to_do() {
    let (mut app, fs) = monday_with_an_overlap();
    ctrl(&mut app, 's');
    let text = screen(&mut app);
    assert!(
        text.contains("ma 2 nov: 0845–1245 overlaps 1230–1300"),
        "{text}"
    );
    assert!(
        text.contains(" Go to day ") && text.contains(" Save anyway ") && text.contains(" Cancel ")
    );
    press(&mut app, KeyCode::Esc);
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some(SAMPLE));
}

#[test]
fn save_anyway_saves_the_inconsistent_day() {
    let (mut app, fs) = monday_with_an_overlap();
    ctrl(&mut app, 's');
    press(&mut app, KeyCode::Char('s'));
    assert!(fs.get(TIMESHEET).unwrap().contains("to: \"12:45\""));
    assert!(!app.has_unsaved_changes());
}

#[test]
fn go_to_day_selects_the_day_and_shows_the_problem() {
    let mut app = sample_with_a_past_open_period();
    ctrl(&mut app, 's');
    let text = screen(&mut app);
    assert!(
        text.contains("di 10 nov: Open period 1230– on a past day"),
        "{text}"
    );
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.selected_day(), date(2026, 11, 10));
    assert_eq!(app.panel(), Panel::Periods);
    let buffer = render(&mut app, 120, 24);
    let rows = rows(&buffer);
    assert!(rows.join("\n").contains("Open period 1230– on a past day"));
    let y = row_index(&rows, "Forgot to stop");
    assert_eq!(buffer[(60, u16::try_from(y).unwrap())].bg, Color::Red);
}

#[test]
fn ctrl_q_with_an_inconsistent_day_asks_before_quitting() {
    let (mut app, _fs) = monday_with_an_overlap();
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Continue);
    let text = screen(&mut app);
    assert!(text.contains(" Quit anyway "));
    // Quit anyway still asks about the unsaved change.
    assert_eq!(press(&mut app, KeyCode::Char('q')), Outcome::Continue);
    assert!(screen(&mut app).contains("Save before quitting?"));
    assert_eq!(press(&mut app, KeyCode::Char('d')), Outcome::Quit);
}

#[test]
fn quit_anyway_without_unsaved_changes_quits() {
    let mut app = sample_with_a_past_open_period();
    ctrl(&mut app, 'q');
    assert_eq!(press(&mut app, KeyCode::Char('q')), Outcome::Quit);
}

#[test]
fn ctrl_s_sorts_every_day_before_saving() {
    let yaml = "days:\n  \"2026-11-02\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n      - { from: \"13:00\", to: \"17:00\", project: null, notes: null, type: work }\n      - { from: \"09:00\", to: \"12:00\", project: null, notes: null, type: work }\n";
    let (mut app, fs) = app_with_fs(yaml, date(2026, 11, 13));
    ctrl(&mut app, 's');
    let saved = fs.get(TIMESHEET).unwrap();
    assert!(
        saved.find("\"09:00\"").unwrap() < saved.find("\"13:00\"").unwrap(),
        "{saved}"
    );
}

#[test]
fn sorting_on_quit_is_not_an_unsaved_change() {
    let yaml = "days:\n  \"2026-11-02\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n      - { from: \"13:00\", to: \"17:00\", project: null, notes: null, type: work }\n      - { from: \"09:00\", to: \"12:00\", project: null, notes: null, type: work }\n";
    let mut app = app_with(yaml, date(2026, 11, 13));
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Quit);
}
