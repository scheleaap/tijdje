mod common;

use common::*;
use jiff::civil::date;
use ratatui::{crossterm::event::KeyCode, style::Color};
use tijdje::app::{App, Screen};

/// The popup's label above its input field: shown only while the popup is open.
const POPUP_LABEL: &str = "Date (YYYY-MM-DD, DD MMM or DD MMM YYYY):";

fn screen(app: &mut App) -> String {
    rows(&render(app, 120, 24)).join("\n")
}

/// The background of the first cell of `text` on the screen. Only for text
/// on a row without wide symbols before it, where a character is a cell.
fn background_of(app: &mut App, text: &str) -> Color {
    let buffer = render(app, 120, 24);
    let rows = rows(&buffer);
    let y = row_index(&rows, text);
    let row = &rows[y];
    let x = row[..row.find(text).unwrap()].chars().count();
    buffer[(u16::try_from(x).unwrap(), u16::try_from(y).unwrap())].bg
}

/// The first day shown in the day list.
fn first_listed_day(app: &mut App) -> String {
    // Below the panel's top border and the header, after "│ ".
    let screen = rows(&render(app, 120, 24));
    screen[2].chars().skip(2).take(9).collect()
}

/// Opens the popup with Ctrl+G, types `text` and presses Enter.
fn go_to(app: &mut App, text: &str) {
    ctrl(app, 'g');
    type_text(app, text);
    press(app, KeyCode::Enter);
}

#[test]
fn ctrl_g_opens_a_popup_with_an_input_field_and_three_buttons() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    ctrl(&mut app, 'g');

    // Then
    let text = screen(&mut app);
    assert!(text.contains("Go to date"), "{text}");
    let buttons = row_containing(&rows(&render(&mut app, 120, 24)), "Today").to_owned();
    assert!(buttons.contains(" Go "), "{buttons}");
    assert!(buttons.contains(" Cancel "), "{buttons}");
}

#[test]
fn the_input_field_shows_what_is_typed() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    ctrl(&mut app, 'g');

    // When
    type_text(&mut app, "15 okt");

    // Then
    assert_eq!(background_of(&mut app, "15 okt"), Color::Cyan);
}

#[test]
fn the_input_field_is_not_highlighted_while_a_button_has_focus() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    ctrl(&mut app, 'g');
    type_text(&mut app, "15 okt");

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(background_of(&mut app, "15 okt"), Color::Gray);
}

#[test]
fn a_typed_date_is_selected() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    go_to(&mut app, "2026-10-14");

    // Then
    assert_eq!(app.selected_day(), date(2026, 10, 14));
    assert!(!screen(&mut app).contains(POPUP_LABEL));
}

#[test]
fn a_date_without_a_year_is_in_the_current_year() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    go_to(&mut app, "3 mrt");

    // Then
    assert_eq!(app.selected_day(), date(2026, 3, 3));
}

#[test]
fn a_date_with_a_year_is_selected() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    go_to(&mut app, "15 dec 2025");

    // Then
    assert_eq!(app.selected_day(), date(2025, 12, 15));
}

#[test]
fn a_weekend_date_selects_the_friday_before() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    go_to(&mut app, "15 nov");

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 13));
}

#[test]
fn the_selected_day_is_shown_after_a_far_jump() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    render(&mut app, 120, 24);

    // When
    go_to(&mut app, "2031-06-11");

    // Then
    let text = screen(&mut app);
    assert!(text.contains("Days: 2031"), "{text}");
    assert!(text.contains("wo 11 jun"), "{text}");
}

#[test]
fn an_invalid_date_shows_a_notification_and_keeps_the_popup_open() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    go_to(&mut app, "15 may");

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 12));
    let text = screen(&mut app);
    assert!(
        text.contains("15 may is not a valid date (YYYY-MM-DD, DD MMM or DD MMM YYYY)"),
        "{text}"
    );
    assert!(text.contains(POPUP_LABEL), "{text}");
    assert!(text.contains("15 may"), "{text}");
}

#[test]
fn the_typed_date_can_be_corrected_after_an_invalid_date() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    go_to(&mut app, "15 may");

    // When
    for _ in 0..3 {
        press(&mut app, KeyCode::Backspace);
    }
    type_text(&mut app, "mei");
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.selected_day(), date(2026, 5, 15));
}

#[test]
fn no_typed_date_asks_for_one() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    go_to(&mut app, "");

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 12));
    assert!(screen(&mut app).contains("Type a date (YYYY-MM-DD, DD MMM or DD MMM YYYY)"));
}

#[test]
fn the_go_button_goes_to_the_typed_date() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    ctrl(&mut app, 'g');
    type_text(&mut app, "2026-10-14");

    // When
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.selected_day(), date(2026, 10, 14));
}

#[test]
fn the_today_button_selects_today() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    go_to(&mut app, "2026-10-14");
    ctrl(&mut app, 'g');

    // When
    press(&mut app, KeyCode::Tab);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 12));
}

#[test]
fn today_on_a_weekend_selects_the_friday_before() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 15));
    go_to(&mut app, "2026-10-14");
    ctrl(&mut app, 'g');

    // When
    alt_key(&mut app, KeyCode::Char('t'));

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 13));
}

#[test]
fn alt_and_a_buttons_first_letter_chooses_it() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    ctrl(&mut app, 'g');
    type_text(&mut app, "2026-10-14");

    // When
    alt_key(&mut app, KeyCode::Char('g'));

    // Then
    assert_eq!(app.selected_day(), date(2026, 10, 14));
}

#[test]
fn letters_are_typed_into_the_input_field_not_taken_as_hotkeys() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    ctrl(&mut app, 'g');

    // When
    type_text(&mut app, "5 aug");
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.selected_day(), date(2026, 8, 5));
}

#[test]
fn esc_and_cancel_close_the_popup_without_moving() {
    for keys in [vec![KeyCode::Esc], vec![KeyCode::BackTab, KeyCode::Enter]] {
        // Given
        let mut app = app_with(SAMPLE, date(2026, 11, 12));
        ctrl(&mut app, 'g');
        type_text(&mut app, "2026-10-14");

        // When
        for &code in &keys {
            press(&mut app, code);
        }

        // Then
        assert_eq!(app.selected_day(), date(2026, 11, 12), "{keys:?}");
        assert!(!screen(&mut app).contains(POPUP_LABEL), "{keys:?}");
    }
}

#[test]
fn other_keys_do_not_reach_the_tracking_screen_while_the_popup_is_open() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    ctrl(&mut app, 'g');

    // When
    ctrl_key(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::F(1));

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 12));
    assert_eq!(app.screen(), Screen::Tracking);
    assert!(screen(&mut app).contains(POPUP_LABEL));
}

#[test]
fn going_to_a_date_checks_the_changed_day_it_leaves() {
    // Given: Monday 2 Nov with its break changed to 1215–1300, which overlaps
    // the 0845–1230 period.
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);

    // When
    go_to(&mut app, "2026-10-14");

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 2));
    let text = screen(&mut app);
    assert!(text.contains("0845–1230 overlaps 1215–1300"), "{text}");
    assert!(!text.contains(POPUP_LABEL), "{text}");
}

#[test]
fn the_today_button_checks_the_changed_day_it_leaves() {
    // Given
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 'g');

    // When
    alt_key(&mut app, KeyCode::Char('t'));

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 2));
}

#[test]
fn ctrl_g_is_ignored_while_editing() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    type_text(&mut app, "09");

    // When
    ctrl(&mut app, 'g');

    // Then
    assert!(app.is_editing());
    assert!(!screen(&mut app).contains(POPUP_LABEL));
}

#[test]
fn ctrl_g_is_ignored_on_the_help_screen() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    press(&mut app, KeyCode::F(1));

    // When
    ctrl(&mut app, 'g');

    // Then
    assert!(!screen(&mut app).contains(POPUP_LABEL));
}

#[test]
fn the_help_screen_lists_ctrl_g() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));

    // When
    press(&mut app, KeyCode::F(1));

    // Then
    let screen = rows(&render(&mut app, 120, 24));
    assert!(row_containing(&screen, "Ctrl+G").contains("Go to date"));
    row_containing(&screen, "Ctrl+Q");
}

#[test]
fn with_a_button_focused_its_plain_letter_chooses_a_button() {
    for (letter, expected) in [
        ('g', date(2026, 10, 14)),
        ('t', date(2026, 11, 12)),
        ('c', date(2026, 11, 9)),
    ] {
        // Given
        let mut app = app_with(SAMPLE, date(2026, 11, 12));
        go_to(&mut app, "2026-11-09");
        ctrl(&mut app, 'g');
        type_text(&mut app, "2026-10-14");
        press(&mut app, KeyCode::Tab);

        // When
        press(&mut app, KeyCode::Char(letter));

        // Then
        assert_eq!(app.selected_day(), expected, "{letter}");
        assert!(!screen(&mut app).contains(POPUP_LABEL), "{letter}");
    }
}

#[test]
fn ctrl_c_copies_the_typed_date() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    let clipboard = FakeClipboard::default();
    app.set_clipboard(Box::new(clipboard.clone()));
    ctrl(&mut app, 'g');
    type_text(&mut app, "15 okt");

    // When
    ctrl(&mut app, 'c');

    // Then
    assert_eq!(clipboard.text().as_deref(), Some("15 okt"));
    assert!(screen(&mut app).contains(POPUP_LABEL));
}

#[test]
fn ctrl_v_replaces_the_typed_date_with_the_clipboard_text() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    app.set_clipboard(Box::new(FakeClipboard::with(" 14 okt\n")));
    ctrl(&mut app, 'g');
    type_text(&mut app, "3 mrt");

    // When
    ctrl(&mut app, 'v');

    // Then
    let text = screen(&mut app);
    assert!(text.contains("│ 14 okt "), "{text}");
    assert!(!text.contains("3 mrt"), "{text}");
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.selected_day(), date(2026, 10, 14));
}

#[test]
fn ctrl_v_with_an_empty_clipboard_shows_a_notification() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    app.set_clipboard(Box::new(FakeClipboard::with("  ")));
    ctrl(&mut app, 'g');
    type_text(&mut app, "3 mrt");

    // When
    ctrl(&mut app, 'v');

    // Then
    let text = screen(&mut app);
    assert!(text.contains("The clipboard is empty"), "{text}");
    assert!(text.contains("3 mrt"), "{text}");
}

#[test]
fn going_to_a_visible_date_scrolls_like_other_day_navigation() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 12));
    let first = first_listed_day(&mut app);

    // When
    go_to(&mut app, "2026-11-18");

    // Then
    assert_eq!(app.selected_day(), date(2026, 11, 18));
    assert_eq!(first_listed_day(&mut app), first);
}

#[test]
fn going_to_a_date_out_of_view_lays_out_the_day_list_as_on_startup() {
    for (target, first) in [("2026-12-15", "ma  7 dec"), ("2026-09-15", "ma  7 sep")] {
        // Given
        let mut app = app_with(SAMPLE, date(2026, 11, 12));
        render(&mut app, 120, 24);

        // When
        go_to(&mut app, target);

        // Then
        assert_eq!(first_listed_day(&mut app), first, "{target}");
    }
}
