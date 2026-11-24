mod common;

use common::*;
use jiff::civil::date;
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, style::Color};
use tijdje::app::{App, Outcome};

const WIDTH: u16 = 120;
const HEIGHT: u16 = 24;
/// The row just above the panels' bottom border, which is above the function-key bar.
const NOTIFICATION_ROW: u16 = HEIGHT - 3;

/// The first and last cell of the run of cyan cells through the middle of
/// the notification row, if the middle cell is cyan.
fn cyan_run_through_the_middle(buffer: &Buffer) -> Option<(u16, u16)> {
    let cyan = |x: &u16| buffer[(*x, NOTIFICATION_ROW)].bg == Color::Cyan;
    let middle = buffer.area.width / 2;
    let first = (0..=middle).rev().take_while(cyan).last();
    let last = (middle..buffer.area.width).take_while(cyan).last();
    first.zip(last)
}

/// The notification's text, if one is shown: the cyan run through the
/// middle of the notification row, when it is centered. Bottom-line values
/// next to it don't count, as the notification has a margin.
fn notification(buffer: &Buffer) -> Option<String> {
    cyan_run_through_the_middle(buffer)
        .filter(|&(first, last)| is_centered(buffer, first, last))
        .map(|(first, last)| {
            (first..=last)
                .map(|x| buffer[(x, NOTIFICATION_ROW)].symbol())
                .collect::<String>()
                .trim()
                .to_owned()
        })
        .filter(|text| !text.is_empty())
}

/// Whether the cells from `first` to `last` are centered on the screen.
fn is_centered(buffer: &Buffer, first: u16, last: u16) -> bool {
    first.abs_diff(buffer.area.width - 1 - last) <= 1
}

fn shown_notification(app: &mut App) -> Option<String> {
    notification(&render(app, WIDTH, HEIGHT))
}

/// Mon 2 Nov of the sample with its first From changed to 0815.
fn app_with_a_change() -> (App, InMemoryFileSystem) {
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    type_text(&mut app, "0815");
    press(&mut app, KeyCode::Enter);
    (app, fs)
}

#[test]
fn saving_shows_saved() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 's');
    assert_eq!(shown_notification(&mut app).as_deref(), Some("Saved"));
}

#[test]
fn saving_on_the_help_screen_shows_saved() {
    let (mut app, _fs) = app_with_a_change();
    press(&mut app, KeyCode::F(1));
    ctrl(&mut app, 's');
    assert_eq!(shown_notification(&mut app).as_deref(), Some("Saved"));
}

#[test]
fn saving_while_quitting_does_not_show_saved() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 'q');
    assert_eq!(press(&mut app, KeyCode::Char('s')), Outcome::Quit);
    assert_eq!(shown_notification(&mut app), None);
}

#[test]
fn the_notification_is_centered_with_a_cyan_background() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 's');
    let buffer = render(&mut app, WIDTH, HEIGHT);
    let (first, last) = cyan_run_through_the_middle(&buffer).expect("cyan cells");
    let text: String = (first..=last)
        .map(|x| buffer[(x, NOTIFICATION_ROW)].symbol())
        .collect();
    assert_eq!(text, " Saved ", "one block");
    assert!(
        is_centered(&buffer, first, last),
        "centered: {first}..={last}"
    );
}

#[test]
fn the_notification_floats_over_the_row_and_leaves_the_rest_visible() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 's');
    let row = &rows(&render(&mut app, WIDTH, HEIGHT))[usize::from(NOTIFICATION_ROW)];
    assert!(row.contains("Saved"), "{row}");
    assert!(row.contains("Worked"), "{row}");
}

#[test]
fn the_notification_disappears_at_the_next_key_press() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 's');
    press(&mut app, KeyCode::Right);
    assert_eq!(shown_notification(&mut app), None);
    let row = &rows(&render(&mut app, WIDTH, HEIGHT))[usize::from(NOTIFICATION_ROW)];
    assert!(row.contains("Worked"), "{row}");
}

#[test]
fn an_invalid_time_is_a_notification() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    type_text(&mut app, "2575");
    press(&mut app, KeyCode::Enter);
    assert_eq!(
        shown_notification(&mut app).as_deref(),
        Some("2575 is not a valid time (HHMM, 0000–2359)")
    );
}

#[test]
fn a_failed_consistency_check_is_a_notification() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);
    ctrl_key(&mut app, KeyCode::Down);
    let text = shown_notification(&mut app).expect("a notification");
    assert!(text.contains("0845–1230 overlaps 1215–1300"), "{text}");
}

#[test]
fn clipboard_messages_are_notifications() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    app.set_clipboard(Box::new(FakeClipboard::with("   ")));
    ctrl(&mut app, 'v');
    assert_eq!(
        shown_notification(&mut app).as_deref(),
        Some("The clipboard is empty")
    );
}

#[test]
fn the_notification_stays_apart_from_the_values_on_the_bottom_line() {
    // Given: on Mon 2 Nov, "7h12m" on the bottom line ends right after where
    // "Nothing to undo" ends.
    let (mut app, _fs) = sample_on(date(2026, 11, 2));

    // When
    ctrl(&mut app, 'z');
    let buffer = render(&mut app, WIDTH, HEIGHT);

    // Then: after the text and its padding cell comes a cell in the panel's
    // colors, and so before it.
    let start = column_of(&buffer, NOTIFICATION_ROW, "Nothing").expect("the notification");
    let end = column_of(&buffer, NOTIFICATION_ROW, "undo").expect("the notification") + 4;
    assert_eq!(buffer[(start - 2, NOTIFICATION_ROW)].bg, Color::Blue);
    assert_eq!(buffer[(end + 1, NOTIFICATION_ROW)].bg, Color::Blue);
}

#[test]
fn nothing_to_undo_and_redo_are_notifications() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    ctrl(&mut app, 'z');
    assert_eq!(
        shown_notification(&mut app).as_deref(),
        Some("Nothing to undo")
    );
    ctrl(&mut app, 'y');
    assert_eq!(
        shown_notification(&mut app).as_deref(),
        Some("Nothing to redo")
    );
}

#[test]
fn the_period_list_ends_above_the_day_note() {
    // Ten consecutive periods take 19 rows: periods with empty rows between them.
    let yaml = hourly_periods_yaml("2026-11-02", 10);
    let mut app = app_with(&yaml, date(2026, 11, 2));
    // The header is on row 1, so the list starts on row 2; below it come the
    // day note's header, its input field and an empty line. The 19 rows don't
    // fit; focusing the last period scrolls it into view at the bottom of the
    // list.
    for _ in 0..9 {
        press(&mut app, KeyCode::Down);
    }
    let rows = rows(&render(&mut app, WIDTH, HEIGHT));
    let last_period = &rows[usize::from(NOTIFICATION_ROW) - 4];
    assert!(last_period.contains("1700 1800"), "{last_period}");
    let day_note_header = &rows[usize::from(NOTIFICATION_ROW) - 3];
    assert!(day_note_header.contains("Day note"), "{day_note_header}");
    let below = &rows[usize::from(NOTIFICATION_ROW)];
    assert!(below.contains("Worked"), "{below}");
}

#[test]
fn overwriting_a_changed_file_while_quitting_does_not_show_saved() {
    let (mut app, fs) = app_with_a_change();
    fs.set(TIMESHEET, "days: {}\n");
    ctrl(&mut app, 'q');
    press(&mut app, KeyCode::Char('s'));
    assert_eq!(press(&mut app, KeyCode::Char('o')), Outcome::Quit);
    assert_eq!(shown_notification(&mut app), None);
}
