mod common;

use common::*;
use ratatui::crossterm::event::KeyCode;
use tijdje::{app::Field, domain::TimeOfDay};

#[test]
fn accepting_a_to_changes_the_next_from() {
    let mut app = app_on_monday();
    tab_to(&mut app, Field::To);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 1).from(), TimeOfDay::new(12, 15));
    let screen = rows(&render(&mut app, 120, 24));
    assert!(row_containing(&screen, "1215 1300").contains("45m"));
}

#[test]
fn pasting_a_to_changes_the_next_from() {
    let mut app = app_on_monday();
    app.set_clipboard(Box::new(FakeClipboard::with("12:15")));
    tab_to(&mut app, Field::To);
    ctrl(&mut app, 'v');
    assert_eq!(period_at(&app, 1).from(), TimeOfDay::new(12, 15));
}

#[test]
fn accepting_a_from_changes_nothing_else() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 1).from(), TimeOfDay::new(12, 15));
    assert_eq!(period_at(&app, 0).to(), TimeOfDay::new(12, 30));
}

#[test]
fn the_chained_from_is_undone_and_redone_with_the_to() {
    let mut app = app_on_monday();
    tab_to(&mut app, Field::To);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 'z');
    assert_eq!(period_at(&app, 0).to(), TimeOfDay::new(12, 30));
    assert_eq!(period_at(&app, 1).from(), TimeOfDay::new(12, 30));
    ctrl(&mut app, 'y');
    assert_eq!(period_at(&app, 0).to(), TimeOfDay::new(12, 15));
    assert_eq!(period_at(&app, 1).from(), TimeOfDay::new(12, 15));
}
