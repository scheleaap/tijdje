mod common;

use common::*;
use jiff::civil::date;
use ratatui::{crossterm::event::KeyCode, style::Color};
use tijdje::app::Outcome;

#[test]
fn left_panel_shows_worked_time_per_day_from_the_file() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 40));
    // Worked examples from the sample file (work periods only, open periods excluded).
    for (day, worked) in [
        ("ma  2 nov", "8h42m"),
        ("di  3 nov", "8h38m"),
        ("wo  4 nov", "8h44m"),
        ("do  5 nov", "8h53m"),
        ("vr  6 nov", "0m"),
        ("ma  9 nov", "7h52m"),
        ("di 10 nov", "3h50m"),
        ("do 12 nov", "4h00m"),
    ] {
        assert!(
            row_containing(&screen, day).contains(worked),
            "{day} should show {worked}"
        );
    }
    let empty_row = row_containing(&screen, "wo 11 nov");
    let worked_column = empty_row.split('│').nth(2).unwrap();
    assert!(
        worked_column.trim().is_empty(),
        "empty day shows no worked time: {empty_row:?}"
    );
}

#[test]
fn only_weekdays_are_listed() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(!screen.contains("za ") && !screen.contains("zo "));
}

#[test]
fn today_is_selected_on_startup() {
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    let buffer = render(&mut app, 120, 24);
    let screen = rows(&buffer);
    let y = row_index(&screen, "do 12 nov");
    let x = screen[y].find("do 12 nov").unwrap();
    assert_eq!(
        buffer[(u16::try_from(x).unwrap() + 1, u16::try_from(y).unwrap())].bg,
        Color::Cyan
    );
    assert_eq!(app.selected_day(), date(2026, 11, 12));
}

#[test]
fn previous_friday_is_selected_on_a_weekend() {
    let app = app_with(SAMPLE, date(2026, 11, 15));
    assert_eq!(app.selected_day(), date(2026, 11, 13));
}

#[test]
fn single_line_separates_weeks() {
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    let screen = rows(&render(&mut app, 120, 24));
    let friday = row_index(&screen, "vr  6 nov");
    assert!(screen[friday + 1].contains("────"));
    assert!(screen[friday + 2].contains("ma  9 nov"));
}

#[test]
fn double_line_separates_years() {
    let mut app = app_with(SAMPLE, date(2026, 12, 31));
    let screen = rows(&render(&mut app, 120, 24));
    let last = row_index(&screen, "do 31 dec");
    assert!(screen[last + 1].contains("════"));
    assert!(screen[last + 2].contains("vr  1 jan"));
    assert!(row_containing(&screen, "vr  1 jan").contains(" 0m "));
}

#[test]
fn ctrl_arrows_select_previous_and_next_weekday() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    ctrl_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 16));
    ctrl_key(&mut app, KeyCode::Up);
    ctrl_key(&mut app, KeyCode::Up);
    assert_eq!(app.selected_day(), date(2026, 11, 12));
}

#[test]
fn ctrl_page_keys_select_previous_and_next_week() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    ctrl_key(&mut app, KeyCode::PageDown);
    assert_eq!(app.selected_day(), date(2026, 11, 18));
    ctrl_key(&mut app, KeyCode::PageUp);
    ctrl_key(&mut app, KeyCode::PageUp);
    assert_eq!(app.selected_day(), date(2026, 11, 4));
}

#[test]
fn list_scrolls_endlessly_and_keeps_the_selection_visible() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    render(&mut app, 120, 24);
    for _ in 0..60 {
        ctrl_key(&mut app, KeyCode::PageUp);
    }
    let screen = rows(&render(&mut app, 120, 24));
    assert_eq!(app.selected_day(), date(2025, 9, 19));
    row_containing(&screen, "vr 19 sep");
    for _ in 0..120 {
        ctrl_key(&mut app, KeyCode::PageDown);
    }
    assert_eq!(app.selected_day(), date(2028, 1, 7));
    let screen = rows(&render(&mut app, 120, 24));
    row_containing(&screen, "vr  7 jan");
}

#[test]
fn ctrl_q_quits() {
    let mut app = app_with(SAMPLE, SAMPLE_TODAY);
    assert_eq!(press(&mut app, KeyCode::Down), Outcome::Continue);
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Quit);
}

#[test]
fn small_terminal_shows_a_message_instead_of_the_ui() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 119, 24)).join("\n");
    assert!(screen.contains("Terminal too small"));
    assert!(!screen.contains("Periods"));
    let screen = rows(&render(&mut app, 120, 23)).join("\n");
    assert!(screen.contains("Terminal too small"));
}

#[test]
fn panels_use_the_norton_commander_palette() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let buffer = render(&mut app, 120, 24);
    assert_eq!(buffer[(60, 10)].bg, Color::Blue);
}

#[test]
fn the_left_panel_title_shows_the_year() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 24));
    assert!(screen[0].contains("Days: 2026"), "{:?}", screen[0]);
}

#[test]
fn the_year_separator_shows_the_new_year() {
    let mut app = app_with(SAMPLE, date(2026, 12, 31));
    let screen = rows(&render(&mut app, 120, 24));
    let separator = &screen[row_index(&screen, "do 31 dec") + 1];
    assert!(separator.contains("═ 2027 ═"), "{separator:?}");
}

#[test]
fn the_left_panel_has_no_column_lines() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 24));
    let row = row_containing(&screen, "ma  9 nov");
    let inside: String = row.chars().skip(1).take(44).collect();
    assert!(!inside.contains('│'), "{row:?}");
}
