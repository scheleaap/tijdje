mod common;

use common::*;
use jiff::civil::date;
use ratatui::{crossterm::event::KeyCode, style::Color};
use tijdje::app::{Field, FieldFocus};

fn focus(period: usize, field: Field) -> FieldFocus {
    FieldFocus { period, field }
}

/// Width of the left panel, in cells.
const LEFT_PANEL_WIDTH: usize = 46;

/// The part of a screen row that belongs to the right panel.
fn right(row: &str) -> String {
    row.chars().skip(LEFT_PANEL_WIDTH).collect()
}

#[test]
fn right_panel_shows_the_periods_of_the_selected_day() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    let screen = rows(&render(&mut app, 120, 24));
    let first = right(row_containing(&screen, "0845"));
    for text in ["0845", "1230", "3h45m", "internal", "Onboarding"] {
        assert!(first.contains(text), "{text} missing in {first:?}");
    }
    let lunch = right(row_containing(&screen, "1230 1300"));
    assert!(lunch.contains("0h30m") && lunch.contains("✓"), "{lunch:?}");
    assert!(screen[0].contains("Periods: ma 2 nov"));
}

#[test]
fn open_period_has_no_duration() {
    let mut app = app_with(SAMPLE, date(2026, 11, 10));
    let screen = rows(&render(&mut app, 120, 24));
    let open = right(row_containing(&screen, "Forgot to stop"));
    assert!(!open.contains("h0") && !open.contains("m "), "{open:?}");
}

#[test]
fn a_gap_between_periods_is_shown_as_an_implied_break_row() {
    let mut app = app_with(SAMPLE, date(2026, 11, 5));
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    let first = row_index(&screen, "0540 1228");
    let implied = row_index(&screen, "1228 1325");
    let second = row_index(&screen, "1325 1530");
    assert_eq!(
        (implied, second),
        (first + 2, first + 4),
        "empty rows in between"
    );
    assert!(
        right(&screen[first + 1])
            .trim_matches(['│', ' '])
            .is_empty()
    );
    let row = right(&screen[implied]);
    assert!(
        row.contains("0h57m") && row.matches('✓').count() == 1,
        "{row:?}"
    );
    // Plain text, not fields.
    let x = u16::try_from(46 + row.find("1228").unwrap()).unwrap();
    assert_eq!(buffer[(x, u16::try_from(implied).unwrap())].bg, Color::Blue);
}

#[test]
fn consecutive_periods_are_separated_by_one_empty_row() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    let screen = rows(&render(&mut app, 120, 24));
    let first = row_index(&screen, "0845 1230");
    assert_eq!(row_index(&screen, "1230 1300"), first + 2);
}

#[test]
fn the_period_warning_is_left_of_from() {
    let mut app = app_with(SAMPLE, date(2026, 11, 5));
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "0540 1228")).starts_with("│❗"));
}

#[test]
fn checkboxes_come_right_after_project_as_colored_boxes() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    let y = row_index(&screen, "0845 1230");
    let row = right(&screen[y]);
    let project_end = row.find("internal").unwrap() + 10;
    let notes = row.find("Onboarding").unwrap();
    let y = u16::try_from(y).unwrap();
    // Four 3-cell boxes, each after a one-cell gap, then Notes.
    for box_start in [
        project_end + 1,
        project_end + 5,
        project_end + 9,
        project_end + 13,
    ] {
        let x = u16::try_from(LEFT_PANEL_WIDTH + box_start).unwrap();
        assert_eq!(buffer[(x, y)].bg, Color::Gray, "box at {box_start}");
    }
    assert_eq!(notes, project_end + 17, "{row:?}");
}

#[test]
fn fields_are_light_gray() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    // Not the first period: its From has focus.
    let y = row_index(&screen, "1300 1757");
    let x = u16::try_from(46 + right(&screen[y]).find("1300").unwrap()).unwrap();
    assert_eq!(buffer[(x, u16::try_from(y).unwrap())].bg, Color::Gray);
}

#[test]
fn unusual_times_show_a_warning() {
    let mut app = app_with(SAMPLE, date(2026, 11, 5));
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "0540 1228")).contains("❗"));
    assert!(!right(row_containing(&screen, "1325 1530")).contains("❗"));
}

#[test]
fn actual_expected_working_time_is_shown_below_the_periods() {
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("Actual expected: 3h36m (regular 7h12m − 3h36m absence)"));

    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("Actual expected: 7h12m"));
    assert!(!screen.contains("regular"));
}

#[test]
fn the_selected_day_has_the_cyan_bar_and_panel_titles_have_no_highlight() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    for title in ["Days", "Periods"] {
        let title_x = screen[0].find(title).unwrap();
        let title_x = u16::try_from(screen[0][..title_x].chars().count()).unwrap();
        assert_eq!(buffer[(title_x, 0)].bg, Color::Blue, "{title}");
    }
    let y = u16::try_from(row_index(&screen, "ma  2 nov ")).unwrap();
    for x in [2, 20] {
        assert_eq!(buffer[(x, y)].bg, Color::Cyan);
    }
}

#[test]
fn from_of_the_first_period_has_focus_at_start() {
    let app = app_with(SAMPLE, date(2026, 11, 2));
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn up_and_down_keep_the_field_and_stop_at_the_edges() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.focus(), focus(1, Field::To));
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.focus(), focus(2, Field::To));
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Up);
    assert_eq!(app.focus(), focus(0, Field::To));
    assert_eq!(
        app.selected_day(),
        date(2026, 11, 2),
        "arrows don't change the day"
    );
}

#[test]
fn tab_follows_the_fields_on_screen_and_continues_in_the_next_period() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    for expected in [
        Field::To,
        Field::Project,
        Field::Break,
        Field::Vacation,
        Field::Sick,
        Field::Holiday,
        Field::Notes,
    ] {
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.focus(), focus(0, expected));
    }
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus(), focus(1, Field::From));
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.focus(), focus(0, Field::Notes));
}

#[test]
fn tab_wraps_around_within_the_day() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.focus(), focus(2, Field::Notes));
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus(), focus(0, Field::From));
    assert_eq!(app.selected_day(), date(2026, 11, 2));
}

#[test]
fn tab_wraps_around_on_the_empty_period_of_a_day_without_periods() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.focus(), focus(0, Field::Notes));
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn up_and_down_skip_implied_break_rows() {
    // Tue 10 Nov: 0810–1200, an implied break 1200–1230, then 1230–.
    let mut app = app_with(SAMPLE, date(2026, 11, 10));
    let screen = rows(&render(&mut app, 120, 24));
    row_containing(&screen, "1200 1230");
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.focus(), focus(1, Field::To));
    press(&mut app, KeyCode::Up);
    assert_eq!(app.focus(), focus(0, Field::To));
}

#[test]
fn ctrl_arrows_select_the_previous_and_next_day_and_focus_from_of_the_first_period() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    press(&mut app, KeyCode::Tab);
    ctrl_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 16));
    assert_eq!(app.focus(), focus(0, Field::From));
    press(&mut app, KeyCode::Tab);
    ctrl_key(&mut app, KeyCode::Up);
    ctrl_key(&mut app, KeyCode::Up);
    assert_eq!(app.selected_day(), date(2026, 11, 12));
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn ctrl_page_keys_select_the_previous_and_next_week() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    ctrl_key(&mut app, KeyCode::PageDown);
    assert_eq!(app.selected_day(), date(2026, 11, 18));
    ctrl_key(&mut app, KeyCode::PageUp);
    ctrl_key(&mut app, KeyCode::PageUp);
    assert_eq!(app.selected_day(), date(2026, 11, 4));
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn the_left_panel_keys_and_alt_navigation_are_removed() {
    let mut app = app_with(SAMPLE, date(2026, 11, 4));
    for code in [
        KeyCode::PageDown,
        KeyCode::PageUp,
        KeyCode::Down,
        KeyCode::Up,
    ] {
        press(&mut app, code);
    }
    for code in [
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Down,
        KeyCode::Up,
        KeyCode::PageDown,
        KeyCode::PageUp,
    ] {
        alt_key(&mut app, code);
    }
    assert_eq!(app.selected_day(), date(2026, 11, 4));
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn focused_field_is_highlighted() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    let y = row_index(&screen, "0845");
    let x = right(&screen[y]).find("0845").unwrap() + LEFT_PANEL_WIDTH;
    let cell = &buffer[(u16::try_from(x).unwrap(), u16::try_from(y).unwrap())];
    assert_eq!(cell.bg, Color::Cyan);
}

#[test]
fn right_panel_scrolls_to_keep_the_focused_period_visible() {
    let periods: String = (0..30)
        .map(|i| {
            format!(
                "      - {{ from: \"{:02}:00\", to: \"{:02}:30\", project: null, notes: \"P{i}\", type: work }}\n",
                i % 24,
                i % 24
            )
        })
        .collect();
    let yaml = format!(
        "days:\n  \"2026-11-02\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n{periods}"
    );
    let mut app = app_with(&yaml, date(2026, 11, 2));
    for _ in 0..29 {
        press(&mut app, KeyCode::Down);
    }
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("P29"));
    assert!(!screen.contains("P0 "));
}

#[test]
fn left_and_right_move_between_fields_and_stop_at_the_period_edges() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    press(&mut app, KeyCode::Right);
    assert_eq!(app.focus(), focus(0, Field::To));
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);
    assert_eq!(
        app.focus(),
        focus(0, Field::From),
        "stops at the first field"
    );
    for _ in 0..10 {
        press(&mut app, KeyCode::Right);
    }
    assert_eq!(
        app.focus(),
        focus(0, Field::Notes),
        "stops at the last field"
    );
}

/// The sample has no periods on Wed 11 Nov.
const DAY_WITHOUT_PERIODS: jiff::civil::Date = date(2026, 11, 11);

#[test]
fn a_day_without_periods_shows_one_empty_period_with_focus_on_from() {
    let mut app = app_with(SAMPLE, DAY_WITHOUT_PERIODS);
    assert_eq!(app.focus(), focus(0, Field::From));
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    let header = row_index(&screen, "From");
    let y = u16::try_from(header + 1).unwrap();
    let from_x =
        u16::try_from(right(&screen[header]).find("From").unwrap() + LEFT_PANEL_WIDTH).unwrap();
    assert_eq!(buffer[(from_x, y)].bg, Color::Cyan, "focused From");
    assert_eq!(buffer[(from_x + 5, y)].bg, Color::Gray, "To field");
    // Moving around stays within the one empty period.
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus(), focus(0, Field::To));
}

#[test]
fn the_empty_period_becomes_a_real_period_once_something_is_filled_in() {
    let mut app = app_with(SAMPLE, DAY_WITHOUT_PERIODS);
    type_text(&mut app, "0900");
    press(&mut app, KeyCode::Enter);
    let periods = &app.timesheet().day(DAY_WITHOUT_PERIODS).periods;
    assert_eq!(periods.len(), 1);
    assert_eq!(periods[0].from, tijdje::domain::TimeOfDay::new(9, 0));
    assert!(app.has_unsaved_changes());
}

#[test]
fn the_empty_period_stays_unsaved_while_nothing_is_filled_in() {
    let mut app = app_with(SAMPLE, DAY_WITHOUT_PERIODS);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    assert!(app.timesheet().day(DAY_WITHOUT_PERIODS).periods.is_empty());
    assert!(!app.has_unsaved_changes(), "accepting nothing is no change");
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn a_period_emptied_again_is_never_saved() {
    let (mut app, fs) = app_with_fs(SAMPLE, DAY_WITHOUT_PERIODS);
    for _ in 0..3 {
        press(&mut app, KeyCode::Tab);
    }
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().day(DAY_WITHOUT_PERIODS).periods.len(), 1);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.focus(), focus(0, Field::Break));
    // The emptied period stays until the day is left, but is never saved.
    assert!(app.timesheet().day(DAY_WITHOUT_PERIODS).is_empty());
    ctrl(&mut app, 's');
    // The sample's open period of 10 Nov fails the check: "Save anyway".
    press(&mut app, KeyCode::Char('s'));
    let saved = fs.get(TIMESHEET).unwrap();
    assert!(!saved.contains("2026-11-11"), "{saved}");
}

/// Clears the focused input field: edit, delete everything, accept.
fn clear_field(app: &mut tijdje::app::App) {
    press(app, KeyCode::Enter);
    for _ in 0..12 {
        press(app, KeyCode::Backspace);
    }
    press(app, KeyCode::Enter);
}

#[test]
fn emptying_every_period_keeps_them_until_the_day_is_left_and_focus_keeps_working() {
    let yaml = "days:\n  \"2026-11-02\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n      - { from: \"09:00\", to: \"10:00\", project: null, notes: null, type: work }\n      - { from: \"10:00\", to: \"11:00\", project: null, notes: null, type: work }\n";
    let mut app = app_with(yaml, date(2026, 11, 2));
    clear_field(&mut app); // period 0 From
    press(&mut app, KeyCode::Right);
    clear_field(&mut app); // period 0 To
    press(&mut app, KeyCode::Down);
    clear_field(&mut app); // period 1 To
    press(&mut app, KeyCode::Left);
    clear_field(&mut app); // period 1 From
    assert_eq!(app.focus(), focus(1, Field::From));
    assert_eq!(app.timesheet().day(date(2026, 11, 2)).periods.len(), 2);
    // The focused field still works.
    type_text(&mut app, "1200");
    press(&mut app, KeyCode::Enter);
    let periods = &app.timesheet().day(date(2026, 11, 2)).periods;
    assert_eq!(periods[1].from, tijdje::domain::TimeOfDay::new(12, 0));
}
