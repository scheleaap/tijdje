mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::{
    app::{App, Field},
    domain::{Period, PeriodType, TimeOfDay},
};

const MONDAY: jiff::civil::Date = date(2026, 11, 2);

/// Mon 2 Nov (0845–1230 work "internal"/"Onboarding", ...), first period
/// focused, with `clipboard`.
fn monday_with(clipboard: &FakeClipboard) -> App {
    let mut app = app_with(SAMPLE, MONDAY);
    app.set_clipboard(Box::new(clipboard.clone()));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    app
}

fn tab_to(app: &mut App, field: Field) {
    while app.focus().map(|f| f.field) != Some(field) {
        press(app, KeyCode::Tab);
    }
}

fn first_period(app: &App) -> &Period {
    &app.timesheet().day(MONDAY).periods[0]
}

fn screen(app: &mut App) -> String {
    rows(&render(app, 120, 24)).join("\n")
}

#[test]
fn ctrl_c_copies_the_focused_field() {
    let clipboard = FakeClipboard::default();
    let mut app = monday_with(&clipboard);
    ctrl(&mut app, 'c');
    assert_eq!(clipboard.text().as_deref(), Some("0845"));
    tab_to(&mut app, Field::Notes);
    ctrl(&mut app, 'c');
    assert_eq!(clipboard.text().as_deref(), Some("Onboarding"));
}

#[test]
fn ctrl_c_while_editing_copies_the_active_input() {
    let clipboard = FakeClipboard::default();
    let mut app = monday_with(&clipboard);
    type_text(&mut app, "07");
    ctrl(&mut app, 'c');
    assert_eq!(clipboard.text().as_deref(), Some("07"));
    assert!(app.is_editing());
}

#[test]
fn ctrl_v_puts_the_clipboard_text_into_the_field_as_accepted_input() {
    let clipboard = FakeClipboard::with("acme");
    let mut app = monday_with(&clipboard);
    tab_to(&mut app, Field::Project);
    ctrl(&mut app, 'v');
    assert!(!app.is_editing());
    assert_eq!(first_period(&app).project.as_deref(), Some("acme"));
}

#[test]
fn ctrl_v_while_editing_replaces_the_active_input() {
    let clipboard = FakeClipboard::with("Workshop");
    let mut app = monday_with(&clipboard);
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, " extra");
    ctrl(&mut app, 'v');
    assert!(!app.is_editing());
    assert_eq!(first_period(&app).notes.as_deref(), Some("Workshop"));
}

#[test]
fn a_pasted_time_may_contain_a_colon() {
    let clipboard = FakeClipboard::with(" 12:35\n");
    let mut app = monday_with(&clipboard);
    ctrl(&mut app, 'v');
    assert_eq!(first_period(&app).from, TimeOfDay::new(12, 35));
}

#[test]
fn an_invalid_pasted_time_is_rejected_with_a_message() {
    let clipboard = FakeClipboard::with("half past nine");
    let mut app = monday_with(&clipboard);
    ctrl(&mut app, 'v');
    assert_eq!(first_period(&app).from, TimeOfDay::new(8, 45));
    assert!(!app.is_editing());
    assert!(screen(&mut app).contains("half past nine is not a valid time"));
}

#[test]
fn an_invalid_pasted_time_keeps_the_active_input() {
    let clipboard = FakeClipboard::with("99:99");
    // (rejected: not a valid time)
    let mut app = monday_with(&clipboard);
    type_text(&mut app, "07");
    ctrl(&mut app, 'v');
    assert!(app.is_editing());
    press(&mut app, KeyCode::Char('1'));
    press(&mut app, KeyCode::Char('5'));
    press(&mut app, KeyCode::Enter);
    assert_eq!(first_period(&app).from, TimeOfDay::new(7, 15));
}

#[test]
fn pasted_lines_are_joined_into_one() {
    let clipboard = FakeClipboard::with("first line\nsecond line");
    let mut app = monday_with(&clipboard);
    tab_to(&mut app, Field::Notes);
    ctrl(&mut app, 'v');
    assert_eq!(
        first_period(&app).notes.as_deref(),
        Some("first line second line")
    );
}

#[test]
fn copying_and_pasting_a_checkbox_does_nothing() {
    let clipboard = FakeClipboard::with("x");
    let mut app = monday_with(&clipboard);
    tab_to(&mut app, Field::Break);
    ctrl(&mut app, 'c');
    ctrl(&mut app, 'v');
    assert_eq!(clipboard.text().as_deref(), Some("x"));
    assert_eq!(first_period(&app).period_type, PeriodType::Work);
}

#[test]
fn a_paste_can_be_undone() {
    let clipboard = FakeClipboard::with("acme");
    let mut app = monday_with(&clipboard);
    tab_to(&mut app, Field::Project);
    ctrl(&mut app, 'v');
    ctrl(&mut app, 'z');
    assert_eq!(first_period(&app).project.as_deref(), Some("internal"));
}

#[test]
fn without_a_clipboard_a_message_says_so() {
    let mut app = monday_with(&FakeClipboard::default());
    app.set_clipboard(Box::new(tijdje::clipboard::NoClipboard));
    ctrl(&mut app, 'v');
    assert!(screen(&mut app).contains("The clipboard is not available"));
}

#[test]
fn a_one_digit_hour_is_padded() {
    let clipboard = FakeClipboard::with("9:35");
    let mut app = monday_with(&clipboard);
    ctrl(&mut app, 'v');
    assert_eq!(first_period(&app).from, TimeOfDay::new(9, 35));
}

#[test]
fn the_message_shows_the_time_as_pasted() {
    let clipboard = FakeClipboard::with("99:99");
    let mut app = monday_with(&clipboard);
    ctrl(&mut app, 'v');
    assert!(screen(&mut app).contains("99:99 is not a valid time"));
}

#[test]
fn an_empty_clipboard_says_so_and_changes_nothing() {
    let clipboard = FakeClipboard::with("   ");
    let mut app = monday_with(&clipboard);
    ctrl(&mut app, 'v');
    assert_eq!(first_period(&app).from, TimeOfDay::new(8, 45));
    assert!(screen(&mut app).contains("The clipboard is empty"));
}
