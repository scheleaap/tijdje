mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
const MONDAY: jiff::civil::Date = date(2026, 11, 2);

use tijdje::{
    app::{App, Field},
    domain::TimeOfDay,
};

/// Mon 2 Nov (0845–1230 work, 1230–1300 break, 1300–1757 work), right panel,
/// first period focused.
fn monday() -> App {
    sample_on(MONDAY).0
}

fn periods(app: &App, day: jiff::civil::Date) -> &[tijdje::domain::Period] {
    &app.timesheet().day(day).periods
}

fn change_first_from(app: &mut App, hhmm: &str) {
    type_text(app, hhmm);
    press(app, KeyCode::Enter);
}

fn screen(app: &mut App) -> String {
    rows(&render(app, 120, 24)).join("\n")
}

#[test]
fn ctrl_z_undoes_and_ctrl_y_redoes_an_accepted_value() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 45));
    ctrl(&mut app, 'y');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 15));
}

#[test]
fn several_changes_are_undone_one_at_a_time() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    change_first_from(&mut app, "0800");
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 15));
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 45));
}

#[test]
fn a_checkbox_toggle_with_its_auto_fill_is_one_step() {
    let (mut app, _fs) = sample_on(date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Char('v'));
    assert_eq!(
        periods(&app, date(2026, 11, 11))[0].from,
        TimeOfDay::new(9, 0)
    );
    ctrl(&mut app, 'z');
    assert!(periods(&app, date(2026, 11, 11)).is_empty());
}

#[test]
fn adding_and_deleting_periods_can_be_undone() {
    let mut app = monday();
    add_period(&mut app);
    assert_eq!(periods(&app, MONDAY).len(), 4);
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY).len(), 3);

    press(&mut app, KeyCode::Up);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Char('d'));
    assert_eq!(periods(&app, MONDAY).len(), 2);
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY).len(), 3);
    assert_eq!(periods(&app, MONDAY)[2].from, TimeOfDay::new(13, 0));
}

#[test]
fn undoing_a_change_on_another_day_selects_that_day() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    ctrl_key(&mut app, KeyCode::Down);
    assert_eq!(app.selected_day(), date(2026, 11, 3));
    ctrl(&mut app, 'z');
    assert_eq!(app.selected_day(), MONDAY);
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 45));
}

#[test]
fn undo_on_the_same_day_keeps_the_focus() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    ctrl(&mut app, 'z');
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn a_new_change_clears_what_can_be_redone() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    ctrl(&mut app, 'z');
    change_first_from(&mut app, "0800");
    ctrl(&mut app, 'y');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 0));
    assert!(screen(&mut app).contains("Nothing to redo"));
}

#[test]
fn nothing_to_undo_says_so() {
    let mut app = monday();
    ctrl(&mut app, 'z');
    assert!(screen(&mut app).contains("Nothing to undo"));
}

#[test]
fn undo_and_redo_are_ignored_while_editing() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    type_text(&mut app, "07");
    ctrl(&mut app, 'z');
    assert!(app.is_editing());
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 15));
}

#[test]
fn saving_keeps_the_history() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    ctrl(&mut app, 's');
    assert!(!app.has_unsaved_changes());
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 45));
    assert!(app.has_unsaved_changes());
}

#[test]
fn sorting_when_leaving_a_day_is_not_an_undo_step() {
    let mut app = monday();
    add_period(&mut app);
    type_text(&mut app, "0700");
    press(&mut app, KeyCode::Tab);
    type_text(&mut app, "0800");
    press(&mut app, KeyCode::Enter);
    ctrl_key(&mut app, KeyCode::Down);
    assert_eq!(
        periods(&app, MONDAY)[0].from,
        TimeOfDay::new(7, 0),
        "sorted"
    );
    // The first undo takes back the last accepted change (To), not the sort.
    ctrl(&mut app, 'z');
    let periods_now = periods(&app, MONDAY);
    assert_eq!(periods_now[3].from, TimeOfDay::new(7, 0));
    assert_eq!(periods_now[3].to, None);
}

#[test]
fn undo_does_nothing_on_the_help_screen() {
    let mut app = monday();
    change_first_from(&mut app, "0815");
    press(&mut app, KeyCode::F(1));
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 15));
}

#[test]
fn clearing_a_field_is_an_undo_step() {
    let mut app = monday();
    press(&mut app, KeyCode::Delete);
    assert_eq!(periods(&app, MONDAY)[0].from, None);
    ctrl(&mut app, 'z');
    assert_eq!(periods(&app, MONDAY)[0].from, TimeOfDay::new(8, 45));
    ctrl(&mut app, 'y');
    assert_eq!(periods(&app, MONDAY)[0].from, None);
}

#[test]
fn clearing_an_empty_field_is_not_an_undo_step() {
    let mut app = monday();
    // The break (period 1) has no project.
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Right);
    assert_eq!(app.focus(), focus(1, Field::Project));
    press(&mut app, KeyCode::Delete);
    ctrl(&mut app, 'z');
    assert!(screen(&mut app).contains("Nothing to undo"));
}
