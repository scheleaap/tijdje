mod common;

use common::*;
use jiff::civil::date;
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, style::Modifier};
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
    app.timesheet().days[&app.selected_day()].periods[0]
        .project
        .as_deref()
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

/// Where `text` starts on row `y`, as a cell column.
fn column_of(buffer: &Buffer, y: u16, text: &str) -> Option<u16> {
    let chars: Vec<String> = text.chars().map(String::from).collect();
    (0..buffer.area.width).find(|&x| {
        chars.iter().enumerate().all(|(i, c)| {
            u16::try_from(i)
                .ok()
                .and_then(|i| x.checked_add(i))
                .filter(|&x| x < buffer.area.width)
                .is_some_and(|x| buffer[(x, y)].symbol() == c)
        })
    })
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
fn the_help_screen_explains_taking_the_suggestion() {
    // Given
    let mut app = app_on(Field::Project);

    // When
    press(&mut app, KeyCode::F(1));

    // Then
    let screen = rows(&render(&mut app, 120, 24));
    assert!(
        row_containing(&screen, "←/→/↑/↓").contains("→ at the end takes the suggestion"),
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
