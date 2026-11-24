mod common;

use common::*;
use jiff::civil::date;
use ratatui::{
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    style::Modifier,
};
use tijdje::app::{App, Field};

/// Mon 2 Nov uses "otto" twice and three other projects once; Tue 3 Nov
/// (today) has one period without a project.
const TIMESHEET_WITH_PROJECTS: &str = r#"schema_version: 1
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "08:00", to: "10:00", project: otto, notes: null, type: work }
      - { from: "10:00", to: "11:00", project: internal, notes: null, type: work }
      - { from: "11:00", to: "12:00", project: otto, notes: null, type: work }
      - { from: "12:00", to: "13:00", project: ottobockxyz, notes: null, type: work }
      - { from: "13:00", to: "14:00", project: "wide日日日日", notes: null, type: work }
  "2026-11-03":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: "10:00", project: null, notes: null, type: work }
"#;

/// As `TIMESHEET_WITH_PROJECTS`, but today's only period is open.
const TIMESHEET_WITH_OPEN_PERIOD: &str = r#"schema_version: 1
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "08:00", to: "10:00", project: otto, notes: null, type: work }
  "2026-11-03":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: null, project: null, notes: null, type: work }
"#;

/// Today's period row starts with these times.
const TODAY_ROW: &str = "0900 1000";

/// Today's first period, focused on `field`.
fn app_on(field: Field) -> App {
    let mut app = app_with(TIMESHEET_WITH_PROJECTS, date(2026, 11, 3));
    while app.focus().field() != Some(field) {
        press(&mut app, KeyCode::Tab);
    }
    app
}

fn project(app: &App) -> Option<&str> {
    period_at(app, 0).project.as_deref()
}

fn notes(app: &App) -> Option<&str> {
    period_at(app, 0).notes.as_deref()
}

/// Today's period has the project `existing`, which Enter starts editing.
fn editing_existing_project(existing: &str) -> App {
    let mut app = app_on(Field::Project);
    type_text(&mut app, existing);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    app
}

/// Whether the screen shows `text` anywhere.
fn shows(app: &mut App, text: &str) -> bool {
    rows(&render(app, 120, 24))
        .iter()
        .any(|row| row.contains(text))
}

/// Today's period row as text.
fn today_row(app: &mut App) -> String {
    row_containing(&rows(&render(app, 120, 24)), TODAY_ROW).to_owned()
}

/// For each character of the ASCII `text` in today's period row, whether it
/// is dimmed. Panics if the row doesn't show `text`.
fn dimmed(app: &mut App, text: &str) -> Vec<bool> {
    let buffer = render(app, 120, 24);
    let y = u16::try_from(row_index(&rows(&buffer), TODAY_ROW)).unwrap();
    let x = column_of(&buffer, y, text).unwrap_or_else(|| panic!("no {text:?} shown"));
    (x..x + u16::try_from(text.len()).unwrap())
        .map(|x| buffer[(x, y)].modifier.contains(Modifier::DIM))
        .collect()
}

#[test]
fn typing_a_project_shows_the_best_earlier_project_as_dimmed_ghost_text() {
    // Given
    let mut app = app_on(Field::Project);

    // When
    type_text(&mut app, "ot");

    // Then
    assert_eq!(dimmed(&mut app, "otto"), [false, false, true, true]);
}

#[test]
fn right_at_the_end_of_the_input_fills_in_the_suggestion_and_keeps_editing() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    press(&mut app, KeyCode::Right);

    // Then
    assert!(app.is_editing());
    assert_eq!(dimmed(&mut app, "otto"), [false; 4]);
}

#[test]
fn the_suggestion_keeps_the_spelling_of_the_earlier_project() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "OT");

    // When
    press(&mut app, KeyCode::Right);

    // Then
    assert_eq!(dimmed(&mut app, "otto"), [false; 4]);
}

#[test]
fn enter_accepts_only_the_typed_text() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(project(&app), Some("ot"));
}

#[test]
fn enter_after_taking_the_suggestion_accepts_it() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");
    press(&mut app, KeyCode::Right);

    // When
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(project(&app), Some("otto"));
}

#[test]
fn tab_takes_the_suggestion_accepts_and_moves_on() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(project(&app), Some("otto"));
    assert!(!app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::Break));
}

#[test]
fn shift_tab_takes_the_suggestion_accepts_and_moves_back() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    press(&mut app, KeyCode::BackTab);

    // Then
    assert_eq!(project(&app), Some("otto"));
    assert!(!app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::To));
}

#[test]
fn ctrl_enter_takes_the_suggestion_accepts_and_moves_on() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(project(&app), Some("otto"));
    assert!(!app.is_editing());
    assert_eq!(app.focus(), focus(1, Field::From));
}

#[test]
fn tab_takes_the_suggestion_in_the_earlier_projects_spelling() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "OT");

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(project(&app), Some("otto"));
}

#[test]
fn tab_with_the_cursor_before_the_end_accepts_only_the_typed_text() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");
    press(&mut app, KeyCode::Left);

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(project(&app), Some("ot"));
}

#[test]
fn tab_in_another_input_field_takes_no_suggestion() {
    // Given
    let mut app = app_on(Field::Notes);
    type_text(&mut app, "ot");

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(notes(&app), Some("ot"));
}

#[test]
fn shift_tab_with_the_cursor_before_the_end_accepts_only_the_typed_text() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");
    press(&mut app, KeyCode::Left);

    // When
    press(&mut app, KeyCode::BackTab);

    // Then
    assert_eq!(project(&app), Some("ot"));
}

#[test]
fn ctrl_enter_with_the_cursor_before_the_end_accepts_only_the_typed_text() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");
    press(&mut app, KeyCode::Left);

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(project(&app), Some("ot"));
}

#[test]
fn ctrl_tab_takes_no_suggestion() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    ctrl_key(&mut app, KeyCode::Tab);

    // Then
    assert!(app.is_editing());
    press(&mut app, KeyCode::Enter);
    assert_eq!(project(&app), Some("ot"));
}

#[test]
fn shift_right_takes_no_suggestion() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    app.handle_key(KeyEvent::new(KeyCode::Right, KeyModifiers::SHIFT));

    // Then
    assert_eq!(dimmed(&mut app, "otto"), [false, false, true, true]);
}

#[test]
fn ctrl_enter_on_the_open_last_period_takes_the_suggestion_and_stays_put() {
    // Given
    let mut app = app_with(TIMESHEET_WITH_OPEN_PERIOD, date(2026, 11, 3));
    tab_to(&mut app, Field::Project);
    type_text(&mut app, "ot");

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(project(&app), Some("otto"));
    assert!(!app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::Project));
    assert!(shows(&mut app, "The last period is still open"));
}

#[test]
fn ctrl_enter_on_the_empty_period_of_a_day_without_periods_takes_the_suggestion() {
    // Given
    let mut app = app_with(TIMESHEET_WITH_PROJECTS, date(2026, 11, 4));
    tab_to(&mut app, Field::Project);
    type_text(&mut app, "ot");

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(project(&app), Some("otto"));
    assert!(!app.is_editing());
    assert!(shows(&mut app, "The last period is still open"));
}

#[test]
fn undo_after_tab_takes_back_the_taken_suggestion_in_one_step() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");
    press(&mut app, KeyCode::Tab);

    // When
    ctrl(&mut app, 'z');

    // Then
    assert_eq!(project(&app), None);
}

#[test]
fn enter_on_an_existing_project_shows_no_suggestion() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "intern");
    press(&mut app, KeyCode::Enter);

    // When
    press(&mut app, KeyCode::Enter);

    // Then
    assert!(app.is_editing());
    assert_eq!(dimmed(&mut app, "intern"), [false; 6]);
    assert!(!shows(&mut app, "internal"));
}

#[test]
fn tab_after_enter_on_an_existing_project_keeps_it() {
    // Given
    let mut app = editing_existing_project("intern");

    // When
    press(&mut app, KeyCode::Tab);

    // Then
    assert_eq!(project(&app), Some("intern"));
}

#[test]
fn typing_after_enter_on_an_existing_project_shows_the_suggestion() {
    // Given
    let mut app = editing_existing_project("intern");

    // When
    type_text(&mut app, "a");

    // Then
    assert_eq!(
        dimmed(&mut app, "internal"),
        [false, false, false, false, false, false, false, true]
    );
}

#[test]
fn deleting_after_enter_on_an_existing_project_shows_the_suggestion() {
    // Given
    let mut app = editing_existing_project("intern");

    // When
    press(&mut app, KeyCode::Backspace);

    // Then
    assert_eq!(
        dimmed(&mut app, "intern"),
        [false, false, false, false, false, true]
    );
}

#[test]
fn moving_the_cursor_after_enter_on_an_existing_project_shows_no_suggestion() {
    // Given
    let mut app = editing_existing_project("intern");

    // When
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Right);

    // Then
    assert!(!shows(&mut app, "internal"));
}

#[test]
fn a_new_project_is_free_text_without_ghost_text() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "acme");

    // When
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(project(&app), Some("acme"));
}

#[test]
fn no_ghost_text_without_a_matching_project() {
    // Given
    let mut app = app_on(Field::Project);

    // When
    type_text(&mut app, "acme");

    // Then
    assert!(today_row(&mut app).contains("acme      "));
}

#[test]
fn no_ghost_text_when_the_cursor_is_not_at_the_end() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");

    // When
    press(&mut app, KeyCode::Left);

    // Then
    assert!(!today_row(&mut app).contains("otto"));
}

#[test]
fn right_before_the_end_of_the_input_only_moves_the_cursor() {
    // Given
    let mut app = app_on(Field::Project);
    type_text(&mut app, "ot");
    press(&mut app, KeyCode::Left);

    // When
    press(&mut app, KeyCode::Right);

    // Then
    assert_eq!(dimmed(&mut app, "otto"), [false, false, true, true]);
}

#[test]
fn ghost_text_is_cut_off_at_the_end_of_the_field() {
    // Given
    let mut app = app_on(Field::Project);

    // When
    type_text(&mut app, "ottobock");

    // Then
    assert_eq!(
        dimmed(&mut app, "ottobockxy"),
        [
            false, false, false, false, false, false, false, false, true, true
        ]
    );
    assert!(!today_row(&mut app).contains("ottobockxyz"));
}

#[test]
fn ghost_text_is_cut_off_by_screen_width_not_by_characters() {
    // Given
    let mut app = app_on(Field::Project);

    // When
    type_text(&mut app, "wi");

    // Then
    // "wi" takes 2 of the field's 10 cells; "de日日日" fills the other 8.
    assert_eq!(today_row(&mut app).matches('日').count(), 3);
}

#[test]
fn other_input_fields_get_no_suggestion() {
    // Given
    let mut app = app_on(Field::Notes);
    type_text(&mut app, "ot");

    // When
    press(&mut app, KeyCode::Right);

    // Then
    assert!(app.is_editing());
    assert!(!today_row(&mut app).contains("otto"));
}

#[test]
fn the_help_window_explains_taking_the_suggestion() {
    // Given
    let mut app = app_on(Field::Project);

    // When
    press(&mut app, KeyCode::F(1));
    press(&mut app, KeyCode::End);

    // Then
    let screen = rows(&render(&mut app, 120, 24));
    assert!(
        row_containing(&screen, "take the project suggestion, accept and move on")
            .contains("Tab/Shift+Tab, Ctrl+Enter"),
        "{screen:#?}"
    );
}

#[test]
fn right_moves_focus_when_not_editing() {
    // Given
    let mut app = app_on(Field::Project);

    // When
    press(&mut app, KeyCode::Right);

    // Then
    assert_eq!(app.focus(), focus(0, Field::Break));
}
