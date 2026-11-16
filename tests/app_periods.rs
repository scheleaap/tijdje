mod common;

use common::*;
use jiff::civil::date;
use ratatui::{crossterm::event::KeyCode, style::Color};
use tijdje::app::{Field, FieldFocus, Panel};

fn focus(period: usize, field: Field) -> Option<FieldFocus> {
    Some(FieldFocus { period, field })
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
    assert!(right(row_containing(&screen, "0540 1228")).starts_with("│⚠️"));
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
    let y = row_index(&screen, "0845 1230");
    let x = u16::try_from(46 + right(&screen[y]).find("0845").unwrap()).unwrap();
    assert_eq!(buffer[(x, u16::try_from(y).unwrap())].bg, Color::Gray);
}

#[test]
fn unusual_times_show_a_warning() {
    let mut app = app_with(SAMPLE, date(2026, 11, 5));
    let screen = rows(&render(&mut app, 120, 24));
    assert!(right(row_containing(&screen, "0540 1228")).contains("⚠️"));
    assert!(!right(row_containing(&screen, "1325 1530")).contains("⚠️"));
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
fn alt_arrows_switch_panels() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    assert_eq!(app.panel(), Panel::Days);
    alt_key(&mut app, KeyCode::Right);
    assert_eq!(app.panel(), Panel::Periods);
    alt_key(&mut app, KeyCode::Left);
    assert_eq!(app.panel(), Panel::Days);
}

#[test]
fn active_panel_title_and_inactive_selection_are_styled() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    let title_x = screen[0].find("Periods").unwrap();
    let title_x = u16::try_from(screen[0][..title_x].chars().count()).unwrap();
    assert_eq!(buffer[(title_x, 0)].bg, Color::Gray);
    let y = u16::try_from(row_index(&screen, "ma  2 nov ")).unwrap();
    // Inactive panel: no bar; the whole row's text is cyan.
    for x in [2, 20] {
        assert_eq!(buffer[(x, y)].fg, Color::Cyan);
        assert_eq!(buffer[(x, y)].bg, Color::Blue);
    }
}

#[test]
fn entering_the_right_panel_focuses_nothing() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    assert_eq!(app.focus(), None);
}

#[test]
fn down_without_focus_focuses_the_first_period_and_up_the_last() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    assert_eq!(app.focus(), focus(0, Field::From));

    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Up);
    assert_eq!(app.focus(), focus(2, Field::From));
}

#[test]
fn up_and_down_keep_the_field_and_stop_at_the_edges() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
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
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Tab);
    assert_eq!(app.focus(), focus(0, Field::From));
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
fn tab_stays_at_the_very_first_and_last_field() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::BackTab);
    assert_eq!(app.focus(), focus(0, Field::From));
    press(&mut app, KeyCode::Up);
    for _ in 0..30 {
        press(&mut app, KeyCode::Tab);
    }
    assert_eq!(app.focus(), focus(2, Field::Notes));
}

#[test]
fn ctrl_arrows_select_another_day_and_clear_focus() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 16));
    assert_eq!(app.focus(), None);
    alt_key(&mut app, KeyCode::Up);
    alt_key(&mut app, KeyCode::Up);
    assert_eq!(app.selected_day(), date(2026, 11, 12));
    assert_eq!(app.panel(), Panel::Periods);
}

#[test]
fn focused_field_is_highlighted() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
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
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Up);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("P29"));
    assert!(!screen.contains("P0 "));
}

#[test]
fn left_and_right_move_between_fields_and_stop_at_the_period_edges() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
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

#[test]
fn alt_page_keys_select_the_previous_and_next_week() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Right);
    alt_key(&mut app, KeyCode::PageDown);
    assert_eq!(app.selected_day(), date(2026, 11, 18));
    alt_key(&mut app, KeyCode::PageUp);
    alt_key(&mut app, KeyCode::PageUp);
    assert_eq!(app.selected_day(), date(2026, 11, 4));
    assert_eq!(app.panel(), Panel::Periods);
}

#[test]
fn ctrl_arrows_no_longer_change_the_day() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Right);
    ctrl_key(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Up);
    ctrl_key(&mut app, KeyCode::Up);
    assert_eq!(app.selected_day(), date(2026, 11, 11));
}
