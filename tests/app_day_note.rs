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

/// Rows of the right panel at 120×24: the period header is on row 1; the
/// day note's header, its input field, an empty line and the bottom line take
/// the last rows inside the panel's frame.
const DAY_NOTE_HEADER_ROW: u16 = 18;
const DAY_NOTE_ROW: u16 = 19;
const EMPTY_ROW: u16 = 20;
const BOTTOM_LINE_ROW: u16 = 21;
/// Where the day note's header and input field start, after the padding.
const DAY_NOTE_X: u16 = LEFT_PADDING_X + 1;
/// Where the day note's input field ends, before the right padding.
const DAY_NOTE_END_X: u16 = RIGHT_PADDING_X - 1;
/// A cell somewhere in the middle of the day note's input field.
const DAY_NOTE_MIDDLE_X: u16 = 80;

#[test]
fn the_day_note_is_at_the_bottom_of_the_right_panel_with_a_header() {
    // Given
    let mut app = monday_with_a_note();

    // When
    let screen = rows(&render(&mut app, 120, 24));

    // Then
    let row = |y: u16| right(&screen[usize::from(y)]);
    assert!(row(1).contains("From"), "{screen:#?}");
    assert!(
        row(DAY_NOTE_HEADER_ROW).starts_with("│ Day note "),
        "{screen:#?}"
    );
    assert!(row(DAY_NOTE_ROW).starts_with("│ Team day "), "{screen:#?}");
    assert_eq!(inside_frame(&row(EMPTY_ROW)), "", "{screen:#?}");
    assert!(row(BOTTOM_LINE_ROW).starts_with("│ "), "{screen:#?}");
    assert!(
        !inside_frame(&row(BOTTOM_LINE_ROW)).is_empty(),
        "{screen:#?}"
    );
}

#[test]
fn the_day_note_header_is_styled_like_the_period_column_headers() {
    // Given
    let mut app = monday_with_a_note();

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    let from_x = column_of(&buffer, 1, "From").unwrap();
    let period_header = &buffer[(from_x, 1)];
    let day_note_header = &buffer[(DAY_NOTE_X, DAY_NOTE_HEADER_ROW)];
    assert_eq!(day_note_header.symbol(), "D");
    assert_eq!(day_note_header.fg, period_header.fg);
    assert_eq!(day_note_header.bg, period_header.bg);
    assert_eq!(day_note_header.modifier, period_header.modifier);
}

#[test]
fn the_day_note_spans_the_full_width_of_the_right_panel_between_the_paddings() {
    // Given
    let mut app = monday_with_a_note();

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    for x in [LEFT_PADDING_X, RIGHT_PADDING_X] {
        assert_eq!(
            buffer[(x, DAY_NOTE_ROW)].bg,
            Color::Blue,
            "padding at x = {x}"
        );
    }
    for x in [DAY_NOTE_X, DAY_NOTE_MIDDLE_X, DAY_NOTE_END_X] {
        assert_eq!(buffer[(x, DAY_NOTE_ROW)].bg, Color::Gray, "x = {x}");
    }
}

#[test]
fn an_empty_day_note_without_focus_shows_no_placeholder() {
    // Given
    let mut app = sample_on(MONDAY).0;

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    let row = right(&rows(&buffer)[usize::from(DAY_NOTE_ROW)]);
    assert_eq!(inside_frame(&row), "", "{row}");
    assert_eq!(buffer[(DAY_NOTE_X, DAY_NOTE_ROW)].bg, Color::Gray);
}

#[test]
fn from_of_the_first_period_has_focus_by_default() {
    let app = monday_with_a_note();
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn down_from_the_last_period_focuses_the_day_note() {
    // Given
    let mut app = sample_on(MONDAY).0;
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.focus(), focus(2, Field::To));

    // When
    press(&mut app, KeyCode::Down);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
    let buffer = render(&mut app, 120, 24);
    assert_eq!(buffer[(DAY_NOTE_MIDDLE_X, DAY_NOTE_ROW)].bg, Color::Cyan);
}

#[test]
fn up_from_the_first_period_keeps_focus() {
    // Given
    let mut app = monday_with_a_note();
    press(&mut app, KeyCode::Right);

    // When
    press(&mut app, KeyCode::Up);

    // Then
    assert_eq!(app.focus(), focus(0, Field::To));
}

#[test]
fn up_from_the_day_note_focuses_from_of_the_last_period() {
    // Given
    let mut app = sample_on(MONDAY).0;
    tab_to(&mut app, Field::Project);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.focus(), DAY_NOTE);

    // When
    press(&mut app, KeyCode::Up);

    // Then
    assert_eq!(app.focus(), focus(2, Field::From));
}

#[test]
fn down_left_and_right_on_the_day_note_keep_focus() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Right);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn the_day_note_is_the_last_field_in_tab_order() {
    // Given
    let mut app = sample_on(MONDAY).0;
    tab_to(&mut app, Field::Notes);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.focus(), focus(2, Field::Notes));

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn tab_wraps_around_from_the_day_note_to_from_of_the_first_period() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn shift_tab_from_the_first_period_wraps_around_to_the_day_note() {
    // Given
    let mut app = sample_on(MONDAY).0;

    // When
    press(&mut app, KeyCode::BackTab);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
}

#[test]
fn shift_tab_from_the_day_note_focuses_notes_of_the_last_period() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::BackTab);

    // Then
    assert_eq!(app.focus(), focus(2, Field::Notes));
}

#[test]
fn ctrl_enter_does_nothing_on_the_day_note() {
    // Given
    let mut app = sample_on(MONDAY).0;
    focus_day_note(&mut app);

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
    assert_eq!(app.timesheet().day(MONDAY).periods.len(), 3);
    assert!(!app.has_unsaved_changes());
}

#[test]
fn ctrl_enter_while_editing_the_day_note_accepts_it_and_keeps_focus() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);
    type_text(&mut app, "Offsite");

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert!(!app.is_editing());
    assert_eq!(day_note(&app, MONDAY), Some("Offsite"));
    assert_eq!(app.focus(), DAY_NOTE);
    assert_eq!(app.timesheet().day(MONDAY).periods.len(), 1);
}

/// Ten hourly periods take more rows than the period list shows at 120×24.
const TEN_PERIODS: usize = 10;

/// Moves focus with ↓ from From of the first period to From of the last
/// of `TEN_PERIODS` periods, and renders, which scrolls the list to it.
fn press_down_to_the_last_of_ten_periods(app: &mut App) {
    for _ in 1..TEN_PERIODS {
        press(app, KeyCode::Down);
    }
    assert_eq!(app.focus(), focus(TEN_PERIODS - 1, Field::From));
    render(app, 120, 24);
}

#[test]
fn focusing_the_day_note_keeps_the_period_list_scrolled() {
    // Given
    let mut app = app_with(&hourly_periods_yaml("2026-11-02", 10), MONDAY);
    press_down_to_the_last_of_ten_periods(&mut app);
    let before = rows(&render(&mut app, 120, 24));

    // When
    press(&mut app, KeyCode::Down);

    // Then
    let screen = rows(&render(&mut app, 120, 24));
    assert_eq!(app.focus(), DAY_NOTE);
    assert_eq!(
        screen[1..usize::from(DAY_NOTE_HEADER_ROW)],
        before[1..usize::from(DAY_NOTE_HEADER_ROW)]
    );
    assert!(
        screen.iter().any(|row| row.contains("1700 1800")),
        "{screen:#?}"
    );
    assert!(
        !screen.iter().any(|row| row.contains("0800 0900")),
        "{screen:#?}"
    );
}

#[test]
fn a_failed_check_with_focus_on_the_day_note_scrolls_to_the_failing_period() {
    // Given: the first period overlaps the second, and the list is scrolled
    // to the last period.
    let yaml = hourly_periods_yaml("2026-11-02", 10).replace(
        r#"from: "08:00", to: "09:00""#,
        r#"from: "08:00", to: "09:30""#,
    );
    let mut app = app_with(&yaml, MONDAY);
    press_down_to_the_last_of_ten_periods(&mut app);
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "Offsite");
    press(&mut app, KeyCode::Enter);
    let before = rows(&render(&mut app, 120, 24));
    assert!(
        !before.iter().any(|row| row.contains("0800 0930")),
        "{before:#?}"
    );

    // When
    ctrl_key(&mut app, KeyCode::Down);

    // Then
    let screen = rows(&render(&mut app, 120, 24));
    assert_eq!(app.selected_day(), MONDAY, "the check failed");
    assert_eq!(app.focus(), DAY_NOTE);
    assert!(
        screen.iter().any(|row| row.contains("0800 0930")),
        "{screen:#?}"
    );
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
    assert_eq!(
        row_index(&screen, "Offsite"),
        usize::from(DAY_NOTE_ROW),
        "active input is shown"
    );
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
    press(&mut app, KeyCode::Up);
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

#[test]
fn the_day_note_is_edited_with_the_shell_like_editing_keys() {
    // Given
    let mut app = monday_with_a_note();
    focus_day_note(&mut app);
    press(&mut app, KeyCode::Enter);

    // When
    ctrl_key(&mut app, KeyCode::Backspace);
    type_text(&mut app, "off");
    press(&mut app, KeyCode::Home);
    type_text(&mut app, "A ");
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(day_note(&app, MONDAY), Some("A Team off"));
}
