mod common;

use common::*;
use jiff::civil::date;
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, style::Color};
use tijdje::app::{App, Outcome};

const WIDTH: u16 = 120;
const HEIGHT: u16 = 24;
/// The row just above the panels' bottom border, which is above the function-key bar.
const NOTIFICATION_ROW: u16 = HEIGHT - 3;

/// The notification: the centered run of cyan cells on the notification row
/// (a selection bar on that row is cyan too, but not centered).
fn notification(buffer: &Buffer) -> Option<String> {
    let width = buffer.area.width;
    let cyan = |x: u16| buffer[(x, NOTIFICATION_ROW)].bg == Color::Cyan;
    let mut x = 0;
    while x < width {
        if !cyan(x) {
            x += 1;
            continue;
        }
        let start = x;
        while x < width && cyan(x) {
            x += 1;
        }
        let end = x - 1;
        if start.abs_diff(width - 1 - end) <= 1 {
            let text: String = (start..=end)
                .map(|x| buffer[(x, NOTIFICATION_ROW)].symbol())
                .collect();
            let text = text.trim();
            return (!text.is_empty()).then(|| text.to_owned());
        }
    }
    None
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
    let cyan: Vec<u16> = (0..WIDTH)
        .filter(|&x| buffer[(x, NOTIFICATION_ROW)].bg == Color::Cyan)
        .collect();
    let (first, last) = (cyan[0], cyan[cyan.len() - 1]);
    assert_eq!(usize::from(last - first + 1), cyan.len(), "one block");
    assert!(
        first.abs_diff(WIDTH - 1 - last) <= 1,
        "centered: {first}..={last}"
    );
}

#[test]
fn the_notification_floats_over_the_row_and_leaves_the_rest_visible() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 's');
    let row = &rows(&render(&mut app, WIDTH, HEIGHT))[usize::from(NOTIFICATION_ROW)];
    assert!(row.contains("Saved"), "{row}");
    assert!(row.contains("Actual"), "{row}");
}

#[test]
fn the_notification_disappears_at_the_next_key_press() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 's');
    press(&mut app, KeyCode::Right);
    assert_eq!(shown_notification(&mut app), None);
    let row = &rows(&render(&mut app, WIDTH, HEIGHT))[usize::from(NOTIFICATION_ROW)];
    assert!(row.contains("Actual expected"), "{row}");
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
    press(&mut app, KeyCode::Tab);
    type_text(&mut app, "1245");
    press(&mut app, KeyCode::Enter);
    ctrl_key(&mut app, KeyCode::Down);
    let text = shown_notification(&mut app).expect("a notification");
    assert!(text.contains("0845–1245 overlaps 1230–1300"), "{text}");
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
fn the_period_list_uses_the_former_message_row() {
    // Ten consecutive periods take 19 rows: periods with empty rows between them.
    let periods: String = (0..10)
        .map(|i| {
            format!(
                "      - {{ from: \"{:02}:00\", to: \"{:02}:00\", project: null, notes: null, type: work }}\n",
                8 + i,
                9 + i
            )
        })
        .collect();
    let yaml = format!(
        "days:\n  \"2026-11-02\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n{periods}"
    );
    let mut app = app_with(&yaml, date(2026, 11, 2));
    let rows = rows(&render(&mut app, WIDTH, HEIGHT));
    // The panel's header is on row 1, so the list starts on row 2.
    let last_period = &rows[usize::from(NOTIFICATION_ROW) - 1];
    assert!(last_period.contains("1700 1800"), "{last_period}");
    let below = &rows[usize::from(NOTIFICATION_ROW)];
    assert!(below.contains("Actual expected"), "{below}");
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
