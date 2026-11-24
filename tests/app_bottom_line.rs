mod common;

use common::*;
use jiff::civil::date;
use ratatui::{buffer::Buffer, style::Color};
use tijdje::app::App;

const TODAY: jiff::civil::Date = date(2026, 11, 13);

fn app_on_today(periods: &[&str]) -> App {
    app_with(&one_day_yaml("2026-11-13", periods), TODAY)
}

/// The bottom line: the last row inside the right panel's frame.
fn bottom_line(buffer: &Buffer) -> String {
    let rows = rows(buffer);
    inside_frame(&right(&rows[rows.len() - 3])).to_owned()
}

#[test]
fn the_bottom_line_shows_worked_time_and_work_until() {
    // Given: the spec's example, 5h30m worked with a 30m implied break.
    let mut app = app_on_today(&[
        r#"{ from: "09:00", to: "12:00", project: acme, notes: null, type: work }"#,
        r#"{ from: "12:30", to: "15:00", project: acme, notes: null, type: work }"#,
    ]);

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    assert_eq!(
        bottom_line(&buffer),
        "Worked 5h30m of 7h12m | Work until 18:00 for 8h"
    );
}

#[test]
fn the_values_on_the_bottom_line_have_a_cyan_background() {
    // Given
    let mut app = app_on_today(&[
        r#"{ from: "09:00", to: "12:00", project: acme, notes: null, type: work }"#,
    ]);

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    for value in ["3h00m", "7h12m", "18:00", "8h"] {
        assert_eq!(
            bottom_line_background(&buffer, value),
            Color::Cyan,
            "{value}"
        );
    }
    for label in ["Worked", "of", "|", "Work until", "for"] {
        assert_eq!(
            bottom_line_background(&buffer, label),
            Color::Blue,
            "{label}"
        );
    }
}

/// The background of the first cell of `text` on the bottom line.
fn bottom_line_background(buffer: &Buffer, text: &str) -> Color {
    let y = buffer.area.height - 3;
    let line: Vec<&str> = (LEFT_PADDING_X..buffer.area.width)
        .map(|x| buffer[(x, y)].symbol())
        .collect();
    let chars: Vec<String> = text.chars().map(String::from).collect();
    let start = line
        .windows(chars.len())
        .position(|window| window == chars.as_slice())
        .expect("text on the bottom line");
    buffer[(LEFT_PADDING_X + u16::try_from(start).unwrap(), y)].bg
}

#[test]
fn the_bottom_line_shows_the_actual_expected_working_time() {
    // Given: Thu 12 Nov from the sample, 4h worked and 3h36m sick.
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    assert!(
        bottom_line(&buffer).starts_with("Worked 4h00m of 3h36m |"),
        "{}",
        bottom_line(&buffer)
    );
}

#[test]
fn work_until_is_a_dash_once_8h_is_worked() {
    // Given: Mon 2 Nov from the sample, 8h42m worked.
    let mut app = app_on_monday();

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    assert_eq!(
        bottom_line(&buffer),
        "Worked 8h42m of 7h12m | Work until – for 8h"
    );
}

#[test]
fn an_empty_day_has_no_work_until() {
    // Given: Wed 11 Nov from the sample has no periods.
    let mut app = app_with(SAMPLE, date(2026, 11, 11));

    // When
    let buffer = render(&mut app, 120, 24);

    // Then
    assert_eq!(
        bottom_line(&buffer),
        "Worked 0m of 7h12m | Work until – for 8h"
    );
}

#[test]
fn durations_under_an_hour_line_up_with_longer_ones() {
    // Given: Mon 2 Nov from the sample, with a 30m break.
    let mut app = app_on_monday();

    // When
    let buffer = render(&mut app, 120, 24);

    // Then: "30m" ends where "3h45m" ends.
    let screen = rows(&buffer);
    let end_of = |row: &str, text: &str| {
        let y = u16::try_from(row_index(&screen, row)).unwrap();
        column_of(&buffer, y, text).unwrap() + u16::try_from(text.len()).unwrap()
    };
    assert_eq!(end_of("1230 1300", "30m "), end_of("0845 1230", "3h45m "));
}

#[test]
fn durations_of_10h_or_more_line_up_with_shorter_ones() {
    // Given
    let mut app = app_on_today(&[
        r#"{ from: "07:00", to: "17:00", project: acme, notes: null, type: work }"#,
        r#"{ from: "17:00", to: "17:30", project: null, notes: null, type: break }"#,
    ]);

    // When
    let buffer = render(&mut app, 120, 24);

    // Then: "30m" ends where "10h00m" ends.
    let screen = rows(&buffer);
    let end_of = |row: &str, text: &str| {
        let y = u16::try_from(row_index(&screen, row)).unwrap();
        column_of(&buffer, y, text).unwrap() + u16::try_from(text.len()).unwrap()
    };
    assert_eq!(end_of("1700 1730", "30m "), end_of("0700 1700", "10h00m "));
}
