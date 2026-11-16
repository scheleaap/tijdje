mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::{
    app::{App, Field, FieldFocus, Outcome},
    domain::{Minutes, TimeOfDay},
};

fn focus(period: usize, field: Field) -> Option<FieldFocus> {
    Some(FieldFocus { period, field })
}

/// Mon 2 Nov from the sample: 0845–1230 work, 1230–1300 break, 1300–1757 work.
fn app_on_monday() -> App {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    app
}

fn period(app: &App, index: usize) -> &tijdje::domain::Period {
    &app.timesheet().days[&app.selected_day()].periods[index]
}

fn tab_to(app: &mut App, field: Field) {
    while app.focus().map(|f| f.field) != Some(field) {
        press(app, KeyCode::Tab);
    }
}

#[test]
fn typing_replaces_the_value_and_enter_accepts_it() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "0815");
    assert!(app.is_editing());
    press(&mut app, KeyCode::Enter);
    assert!(!app.is_editing());
    assert_eq!(period(&app, 0).from, TimeOfDay::new(8, 15));
    let screen = rows(&render(&mut app, 120, 24));
    // 0815–1230 (4h15m) + 1300–1757 (4h57m)
    assert!(row_containing(&screen, "ma  2 nov").contains("9h12m"));
    assert!(row_containing(&screen, "0815 1230").contains("4h15m"));
}

#[test]
fn active_input_is_shown_while_editing() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "07");
    let screen = rows(&render(&mut app, 120, 24));
    assert!(
        row_containing(&screen, "07   1230").contains("3h45m"),
        "{screen:#?}"
    );
}

#[test]
fn esc_rejects_the_active_input() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "07");
    press(&mut app, KeyCode::Esc);
    assert!(!app.is_editing());
    assert_eq!(period(&app, 0).from, TimeOfDay::new(8, 45));
}

#[test]
fn cursor_keys_edit_within_the_active_input() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "abc");
    press(&mut app, KeyCode::Home);
    type_text(&mut app, "X");
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::End);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);
    // "abc" → "Xabc" → delete "b" → "Xac" → backspace before "c" removes "a" → "Xc"
    assert_eq!(period(&app, 0).notes.as_deref(), Some("Xc"));
}

#[test]
fn time_fields_take_at_most_four_digits() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1a2:345");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period(&app, 0).from, TimeOfDay::new(12, 34));
}

#[test]
fn typing_a_letter_in_a_time_field_does_not_start_editing() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "x");
    assert!(!app.is_editing());
}

#[test]
fn invalid_time_keeps_editing_and_shows_a_message() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "2575");
    press(&mut app, KeyCode::Enter);
    assert!(app.is_editing());
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("2575 is not a valid time"));
    press(&mut app, KeyCode::Backspace);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(
        !screen.contains("not a valid time"),
        "message disappears at the next key"
    );
    press(&mut app, KeyCode::Esc);
    assert_eq!(period(&app, 0).from, TimeOfDay::new(8, 45));
}

#[test]
fn incomplete_time_is_invalid() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "915");
    press(&mut app, KeyCode::Enter);
    assert!(app.is_editing());
}

#[test]
fn an_empty_value_can_be_accepted() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1");
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);
    assert_eq!(period(&app, 0).from, None);

    tab_to(&mut app, Field::Project);
    type_text(&mut app, "x");
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);
    assert_eq!(period(&app, 0).project, None);
}

#[test]
fn tab_accepts_and_moves_on() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Project);
    type_text(&mut app, "acme");
    press(&mut app, KeyCode::Tab);
    assert!(!app.is_editing());
    assert_eq!(period(&app, 0).project.as_deref(), Some("acme"));
    assert_eq!(app.focus(), focus(0, Field::Break));
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "n");
    press(&mut app, KeyCode::BackTab);
    assert_eq!(period(&app, 0).notes.as_deref(), Some("n"));
    assert_eq!(app.focus(), focus(0, Field::Holiday));
}

#[test]
fn tab_with_an_invalid_time_stays_in_the_field() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "99");
    press(&mut app, KeyCode::Tab);
    assert!(app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn other_shortcuts_are_ignored_while_editing() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "07");
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Left);
    alt_key(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::F(1));
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Continue);
    assert!(app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::From));
    assert_eq!(app.selected_day(), date(2026, 11, 2));
    assert_eq!(app.panel(), tijdje::app::Panel::Periods);
    assert_eq!(app.screen(), tijdje::app::Screen::Tracking);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(!screen.contains("─ Delete period ─"));
}

#[test]
fn enter_in_the_last_period_adds_a_period_starting_at_its_end() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Up);
    tab_to(&mut app, Field::Notes);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 4);
    assert_eq!(period(&app, 3).from, TimeOfDay::new(17, 57));
    assert_eq!(period(&app, 3).to, None);
    assert_eq!(app.focus(), focus(3, Field::From));
}

#[test]
fn enter_without_focus_adds_a_period_too() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.focus(), focus(3, Field::From));
}

#[test]
fn enter_in_an_earlier_period_does_nothing() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 3);
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn enter_on_an_empty_day_adds_its_first_period() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Enter);
    let day = &app.timesheet().days[&date(2026, 11, 11)];
    assert_eq!(day.periods.len(), 1);
    assert_eq!(day.periods[0].from, None);
    assert_eq!(day.regular_expected_working_time(), Minutes(432));
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn ctrl_del_deletes_the_focused_period_after_confirming() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Tab);
    ctrl_key(&mut app, KeyCode::Delete);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(
        screen.contains("Delete period 0845–1230?")
            && screen.contains(" Delete ")
            && screen.contains(" Cancel ")
    );
    press(&mut app, KeyCode::Char('d'));
    let periods = &app.timesheet().days[&date(2026, 11, 2)].periods;
    assert_eq!(periods.len(), 2);
    assert_eq!(
        periods[0].from,
        TimeOfDay::new(12, 30),
        "the break took its place"
    );
    assert_eq!(app.focus(), focus(0, Field::To));
}

#[test]
fn delete_popup_can_be_canceled() {
    for keys in [
        vec![KeyCode::Esc],
        vec![KeyCode::Char('c')],
        vec![KeyCode::Right, KeyCode::Enter],
    ] {
        let mut app = app_on_monday();
        press(&mut app, KeyCode::Down);
        ctrl_key(&mut app, KeyCode::Delete);
        for key in keys {
            press(&mut app, key);
        }
        assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 3);
        let screen = rows(&render(&mut app, 120, 24)).join("\n");
        assert!(!screen.contains("─ Delete period ─"));
    }
}

#[test]
fn delete_popup_buttons_are_chosen_with_arrows_and_enter() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 2);
}

#[test]
fn deleting_the_last_period_focuses_the_previous_one() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Up);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Char('d'));
    assert_eq!(app.focus(), focus(1, Field::From));
}

#[test]
fn deleting_the_only_period_leaves_no_focus() {
    let mut app = app_with(SAMPLE, date(2026, 11, 6));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Char('d'));
    assert!(app.timesheet().days[&date(2026, 11, 6)].periods.is_empty());
    assert_eq!(app.focus(), None);
}

#[test]
fn ctrl_del_without_focus_does_nothing() {
    let mut app = app_on_monday();
    ctrl_key(&mut app, KeyCode::Delete);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(!screen.contains("─ Delete period ─"));
}

#[test]
fn long_notes_are_cut_off_and_scroll_while_editing() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Notes);
    let long = "the quick brown fox jumps over the lazy dog";
    type_text(&mut app, long);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(
        screen.contains("lazy dog"),
        "the end of the text is visible while typing"
    );
    assert!(!screen.contains("the quick"));
    press(&mut app, KeyCode::Enter);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("the quick brown"));
    assert!(screen.contains('…'));
    assert!(!screen.contains("lazy dog"));
}

#[test]
fn typing_on_a_checkbox_does_nothing() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Break);
    type_text(&mut app, "x");
    assert!(!app.is_editing());
}

#[test]
fn the_cursor_is_shown_in_the_edited_field() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "07");
    let mut terminal = ratatui::Terminal::new(ratatui::backend::TestBackend::new(120, 24)).unwrap();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let screen = rows(terminal.backend().buffer());
    let y = row_index(&screen, "07   1230");
    let x = screen[y].chars().position(|c| c == '7').unwrap() + 1;
    let cursor = terminal.get_cursor_position().unwrap();
    assert_eq!(
        (cursor.x, cursor.y),
        (u16::try_from(x).unwrap(), u16::try_from(y).unwrap())
    );
}

#[test]
fn enter_does_not_add_a_period_after_an_empty_last_period() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 11)].periods.len(), 1);
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn enter_does_not_add_a_period_after_an_open_last_period() {
    // Tue 10 Nov from the sample: 0810–1200, then 1230– (open).
    let mut app = app_with(SAMPLE, date(2026, 11, 10));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 10)].periods.len(), 2);
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 10)].periods.len(), 2);
}

#[test]
fn popups_have_a_short_title_and_a_message() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Delete);
    let screen = rows(&render(&mut app, 120, 24));
    let title = row_index(&screen, "─ Delete period ─");
    let message = row_index(&screen, "Delete period 0845–1230?");
    assert!(message > title, "the message is inside, below the title");
}

#[test]
fn the_consistency_popup_has_a_short_title() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Tab);
    type_text(&mut app, "1245");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 's');
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("─ Consistency ─"), "{screen}");
    assert!(
        screen.contains("ma 2 nov: 0845–1245 overlaps 1230–1300"),
        "{screen}"
    );
}

#[test]
fn left_and_right_move_the_cursor_while_editing() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "ac");
    press(&mut app, KeyCode::Left);
    type_text(&mut app, "b");
    press(&mut app, KeyCode::Right);
    type_text(&mut app, "d");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period(&app, 0).notes.as_deref(), Some("abcd"));
    assert_eq!(app.focus(), focus(0, Field::Notes));
}
