mod common;

use common::*;
use jiff::civil::{Date, date};
use ratatui::{crossterm::event::KeyCode, style::Color};
use tijdje::app::{App, Field};

const MONDAY: Date = date(2026, 11, 2);
/// A day without periods in the sample.
const EMPTY_WEDNESDAY: Date = date(2026, 11, 11);

/// Mon 2 Nov from the sample (0845–1230, 1230–1300, 1300–1757), with
/// `clipboard`.
fn monday_with(clipboard: &FakeClipboard) -> App {
    let mut app = sample_on(MONDAY).0;
    app.set_clipboard(Box::new(clipboard.clone()));
    app
}

/// Mon 2 Nov with the day note "Team day" and one period.
fn monday_with_a_note() -> App {
    let yaml = "days:\n  \"2026-11-02\":\n    regular_expected_working_time_in_minutes: 432\n    notes: \"Team day\"\n    periods:\n      - { from: \"09:00\", to: \"12:00\", project: null, notes: null, type: work }\n";
    app_with(yaml, MONDAY)
}

fn day_note(app: &App, date: Date) -> Option<&str> {
    app.timesheet().day(date).day_note.as_deref()
}

#[test]
fn the_day_note_is_shown_above_the_period_header() {
    // Given
    let mut app = monday_with_a_note();

    // When
    let screen = rows(&render(&mut app, 120, 24));

    // Then
    assert_eq!(row_index(&screen, "Team day"), 1, "{screen:#?}");
    assert_eq!(row_index(&screen, "From"), 2, "{screen:#?}");
}

#[test]
fn the_day_note_spans_the_full_width_of_the_right_panel() {
    // Given
    let mut app = monday_with_a_note();

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    for x in [47, 80, 118] {
        assert_eq!(buffer[(x, 1)].bg, Color::Gray, "x = {x}");
    }
}

#[test]
fn an_empty_day_note_without_focus_shows_a_dimmed_placeholder() {
    // Given
    let mut app = sample_on(MONDAY).0;

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    assert_eq!(row_index(&rows(&buffer), "Day note"), 1);
    let x = 47 + u16::try_from(right_panel(&rows(&buffer)[1]).find("Day note").unwrap()).unwrap();
    assert_eq!(buffer[(x, 1)].fg, Color::DarkGray);
    assert_eq!(buffer[(x, 1)].bg, Color::Gray);
}

#[test]
fn an_empty_day_note_with_focus_shows_no_placeholder() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);

    // When
    let screen = rows(&render(&mut app, 120, 24));

    // Then
    assert!(!screen[1].contains("Day note"), "{}", screen[1]);
}

#[test]
fn from_of_the_first_period_has_focus_by_default() {
    let app = monday_with_a_note();
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn up_from_the_first_period_focuses_the_day_note() {
    // Given
    let mut app = monday_with_a_note();
    press(&mut app, KeyCode::Right);

    // When
    press(&mut app, KeyCode::Up);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
    let buffer = render(&mut app, 120, 24);
    assert_eq!(buffer[(60, 1)].bg, Color::Cyan);
}

#[test]
fn up_left_and_right_on_the_day_note_keep_focus() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Right);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn down_from_the_day_note_focuses_from_of_the_first_period() {
    // Given
    let mut app = monday_with_a_note();
    press(&mut app, KeyCode::Right);
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Down);

    // Then
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn the_day_note_is_the_first_field_in_tab_order() {
    // Given
    let mut app = sample_on(MONDAY).0;

    // When
    press(&mut app, KeyCode::BackTab);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn tab_wraps_around_from_the_last_field_to_the_day_note() {
    // Given
    let mut app = sample_on(MONDAY).0;
    press(&mut app, KeyCode::BackTab);
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.focus(), focus(2, Field::Notes));

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn ctrl_enter_on_the_day_note_focuses_from_of_the_first_period() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), focus(0, Field::From));
    assert_eq!(app.timesheet().day(MONDAY).periods.len(), 3);
}

#[test]
fn typing_replaces_the_day_note_and_enter_accepts_it() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);

    // When
    type_text(&mut app, "Offsite");
    let screen = rows(&render(&mut app, 120, 24));
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(row_index(&screen, "Offsite"), 1, "active input is shown");
    assert!(!app.is_editing());
    assert_eq!(day_note(&app, MONDAY), Some("Offsite"));
    assert!(app.has_unsaved_changes());
}

#[test]
fn enter_edits_the_day_note_with_its_existing_text() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "!");
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(day_note(&app, MONDAY), Some("Team day!"));
}

#[test]
fn esc_rejects_the_active_input_of_the_day_note() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);
    type_text(&mut app, "Offsite");

    // When
    press(&mut app, KeyCode::Esc);

    // Then
    assert_eq!(day_note(&app, MONDAY), Some("Team day"));
}

#[test]
fn a_day_note_of_only_spaces_is_no_day_note() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);

    // When
    type_text(&mut app, "   ");
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(day_note(&app, MONDAY), None);
}

#[test]
fn a_pasted_day_note_of_only_spaces_and_line_breaks_is_no_day_note() {
    // Given
    let clipboard = FakeClipboard::with("  \n  ");
    let mut app = monday_with_a_note();
    app.set_clipboard(Box::new(clipboard.clone()));
    focus_day_note(&mut app);

    // When
    ctrl(&mut app, 'v');

    // Then
    assert_eq!(day_note(&app, MONDAY), None);
    assert!(!app.is_editing());
}

#[test]
fn del_clears_the_day_note() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Delete);

    // Then
    assert_eq!(day_note(&app, MONDAY), None);
}

#[test]
fn ctrl_c_copies_the_day_note() {
    // Given
    let clipboard = FakeClipboard::default();
    let mut app = monday_with_a_note();
    app.set_clipboard(Box::new(clipboard.clone()));
    focus_day_note(&mut app);

    // When
    ctrl(&mut app, 'c');

    // Then
    assert_eq!(clipboard.text().as_deref(), Some("Team day"));
}

#[test]
fn ctrl_v_pastes_into_the_day_note_on_one_line() {
    // Given
    let clipboard = FakeClipboard::with("Team\nday");
    let mut app = monday_with(&clipboard);
    focus_day_note(&mut app);

    // When
    ctrl(&mut app, 'v');

    // Then
    assert!(!app.is_editing());
    assert_eq!(day_note(&app, MONDAY), Some("Team day"));
}

#[test]
fn period_keys_do_nothing_on_the_day_note() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);
    let before = app.timesheet().clone();

    // When
    alt_key(&mut app, KeyCode::Char('b'));
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Esc);

    // Then
    assert_eq!(app.timesheet(), &before);
    assert!(!app.has_unsaved_changes());
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn alt_f_does_nothing_on_the_day_note() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);
    let before = app.timesheet().clone();

    // When
    alt_key(&mut app, KeyCode::Char('f'));

    // Then
    assert_eq!(app.timesheet(), &before);
    assert!(!app.has_unsaved_changes());
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn changing_the_day_note_is_an_undo_step() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);
    type_text(&mut app, "Offsite");
    press(&mut app, KeyCode::Enter);

    // When
    ctrl(&mut app, 'z');

    // Then
    assert_eq!(day_note(&app, MONDAY), Some("Team day"));
    ctrl(&mut app, 'y');
    assert_eq!(day_note(&app, MONDAY), Some("Offsite"));
}

#[test]
fn clearing_the_day_note_is_an_undo_step() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);
    press(&mut app, KeyCode::Delete);

    // When
    ctrl(&mut app, 'z');

    // Then
    assert_eq!(day_note(&app, MONDAY), Some("Team day"));
}

#[test]
fn a_day_with_only_a_note_is_saved_and_still_shows_the_empty_period() {
    // Given
    let (mut app, fs) = sample_on(EMPTY_WEDNESDAY);
    focus_day_note(&mut app);
    type_text(&mut app, "Dentist");
    press(&mut app, KeyCode::Enter);

    // When
    ctrl(&mut app, 's');

    // Then
    let yaml = fs.get(TIMESHEET).unwrap();
    assert!(
        yaml.contains("  \"2026-11-11\":\n    regular_expected_working_time_in_minutes: 432\n    notes: \"Dentist\"\n    periods: []\n"),
        "{yaml}"
    );
    assert_eq!(app.timesheet().day(EMPTY_WEDNESDAY).periods.len(), 0);
    press(&mut app, KeyCode::Down);
    let buffer = render(&mut app, 120, 24);
    let first_period_row = u16::try_from(row_index(&rows(&buffer), "From") + 1).unwrap();
    assert_eq!(
        buffer[(50, first_period_row)].bg,
        Color::Cyan,
        "the empty period's From has focus"
    );
}

#[test]
fn saving_keeps_focus_on_the_day_note() {
    // Given
    let (mut app, _fs) = sample_on(EMPTY_WEDNESDAY);
    focus_day_note(&mut app);
    type_text(&mut app, "Offsite");
    press(&mut app, KeyCode::Enter);

    // When
    ctrl(&mut app, 's');

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn switching_days_from_the_day_note_focuses_from_of_the_first_period() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);

    // When
    ctrl_key(&mut app, KeyCode::Down);

    // Then
    assert_eq!(app.focus(), focus(0, Field::From));
}

/// The part of a screen row that belongs to the right panel.
fn right_panel(row: &str) -> String {
    row.chars().skip(46).collect()
}
