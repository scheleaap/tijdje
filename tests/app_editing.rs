mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::{
    app::{App, Field, FieldFocus, Outcome},
    domain::{Minutes, TimeOfDay},
};

fn focus(period: usize, field: Field) -> FieldFocus {
    FieldFocus { period, field }
}

#[test]
fn typing_replaces_the_value_and_enter_accepts_it() {
    let mut app = app_on_monday();
    type_text(&mut app, "0815");
    assert!(app.is_editing());
    press(&mut app, KeyCode::Enter);
    assert!(!app.is_editing());
    assert_eq!(period_at(&app, 0).from, TimeOfDay::new(8, 15));
    let screen = rows(&render(&mut app, 120, 24));
    // 0815–1230 (4h15m) + 1300–1757 (4h57m)
    assert!(row_containing(&screen, "ma  2 nov").contains("9h12m"));
    assert!(row_containing(&screen, "0815 1230").contains("4h15m"));
}

#[test]
fn active_input_is_shown_while_editing() {
    let mut app = app_on_monday();
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
    type_text(&mut app, "07");
    press(&mut app, KeyCode::Esc);
    assert!(!app.is_editing());
    assert_eq!(period_at(&app, 0).from, TimeOfDay::new(8, 45));
}

#[test]
fn cursor_keys_edit_within_the_active_input() {
    let mut app = app_on_monday();
    // The break (period 1) has no notes yet.
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
    assert_eq!(period_at(&app, 1).notes.as_deref(), Some("Xc"));
}

#[test]
fn time_fields_take_at_most_four_digits() {
    let mut app = app_on_monday();
    type_text(&mut app, "1a2:345");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 0).from, TimeOfDay::new(12, 34));
}

#[test]
fn typing_a_letter_in_a_time_field_does_not_start_editing() {
    let mut app = app_on_monday();
    type_text(&mut app, "x");
    assert!(!app.is_editing());
}

#[test]
fn invalid_time_keeps_editing_and_shows_a_message() {
    let mut app = app_on_monday();
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
    assert_eq!(period_at(&app, 0).from, TimeOfDay::new(8, 45));
}

#[test]
fn incomplete_time_is_invalid() {
    let mut app = app_on_monday();
    type_text(&mut app, "915");
    press(&mut app, KeyCode::Enter);
    assert!(app.is_editing());
}

#[test]
fn an_empty_value_can_be_accepted() {
    let mut app = app_on_monday();
    type_text(&mut app, "1");
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 0).from, None);

    tab_to(&mut app, Field::Project);
    type_text(&mut app, "x");
    press(&mut app, KeyCode::Backspace);
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 0).project, None);
}

#[test]
fn tab_accepts_and_moves_on() {
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Project);
    type_text(&mut app, "acme");
    press(&mut app, KeyCode::Tab);
    assert!(!app.is_editing());
    assert_eq!(period_at(&app, 0).project.as_deref(), Some("acme"));
    assert_eq!(app.focus(), focus(0, Field::Break));
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "n");
    press(&mut app, KeyCode::BackTab);
    assert_eq!(period_at(&app, 0).notes.as_deref(), Some("n"));
    assert_eq!(app.focus(), focus(0, Field::Holiday));
}

#[test]
fn ctrl_enter_accepts_and_moves_to_from_of_the_next_period() {
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Project);
    type_text(&mut app, "acme");
    ctrl_key(&mut app, KeyCode::Enter);
    assert!(!app.is_editing());
    assert_eq!(period_at(&app, 0).project.as_deref(), Some("acme"));
    assert_eq!(app.focus(), focus(1, Field::From));
}

#[test]
fn ctrl_enter_with_an_invalid_time_stays_in_the_field() {
    let mut app = app_on_monday();
    type_text(&mut app, "99");
    ctrl_key(&mut app, KeyCode::Enter);
    assert!(app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::From));
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 3);
}

#[test]
fn tab_with_an_invalid_time_stays_in_the_field() {
    let mut app = app_on_monday();
    type_text(&mut app, "99");
    press(&mut app, KeyCode::Tab);
    assert!(app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn other_shortcuts_are_ignored_while_editing() {
    let mut app = app_on_monday();
    type_text(&mut app, "07");
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::PageDown);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::F(1));
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Continue);
    ctrl(&mut app, 's');
    ctrl(&mut app, 'z');
    ctrl(&mut app, 'y');
    assert!(app.is_editing());
    assert!(!app.has_unsaved_changes());
    assert_eq!(app.focus(), focus(0, Field::From));
    assert_eq!(app.selected_day(), date(2026, 11, 2));
    assert_eq!(app.screen(), tijdje::app::Screen::Tracking);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(!screen.contains("─ Delete period ─"));
    assert_eq!(screen.matches("Nothing to").count(), 0, "silently");
}

#[test]
fn ctrl_enter_moves_to_from_of_the_next_period() {
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Notes);
    ctrl_key(&mut app, KeyCode::Enter);
    assert_eq!(app.focus(), focus(1, Field::From));
    ctrl_key(&mut app, KeyCode::Enter);
    assert_eq!(app.focus(), focus(2, Field::From));
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 3);
}

#[test]
fn ctrl_enter_in_the_last_period_adds_a_period_starting_where_it_ends() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Notes);
    ctrl_key(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 4);
    assert_eq!(period_at(&app, 3).from, TimeOfDay::new(17, 57));
    assert_eq!(period_at(&app, 3).to, None);
    assert_eq!(app.focus(), focus(3, Field::From));
}

#[test]
fn removed_alt_keys_do_nothing() {
    let mut app = app_on_monday();
    for c in ['p', 'n'] {
        alt_key(&mut app, KeyCode::Char(c));
        assert_eq!(app.focus(), focus(0, Field::From), "Alt+{c}");
    }
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Enter);
    alt_key(&mut app, KeyCode::Delete);
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 3);
    assert!(!app.is_editing());
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(!screen.contains("─ Delete period ─"));
}

#[test]
fn enter_no_longer_adds_a_period() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Up);
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Esc);
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 3);
}

#[test]
fn enter_starts_editing_with_the_existing_text() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Enter);
    assert!(app.is_editing());
    press(&mut app, KeyCode::Backspace);
    type_text(&mut app, "0");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 0).from, TimeOfDay::new(8, 40));
}

#[test]
fn typing_in_notes_replaces_the_value_too() {
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "Day 1");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 0).notes.as_deref(), Some("Day 1"));
}

#[test]
fn typing_in_other_fields_replaces_the_value() {
    let mut app = app_on_monday();
    tab_to(&mut app, Field::Project);
    type_text(&mut app, "acme");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 0).project.as_deref(), Some("acme"));
}

#[test]
fn filling_in_the_empty_period_of_a_day_without_periods_adds_its_first_period() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    ctrl_key(&mut app, KeyCode::Enter);
    assert!(
        app.timesheet().day(date(2026, 11, 11)).periods.is_empty(),
        "Ctrl+Enter does nothing on the empty period"
    );
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(!screen.contains("still open"), "not even a notification");
    type_text(&mut app, "0900");
    press(&mut app, KeyCode::Enter);
    let day = &app.timesheet().days[&date(2026, 11, 11)];
    assert_eq!(day.periods.len(), 1);
    assert_eq!(day.periods[0].from, TimeOfDay::new(9, 0));
    assert_eq!(day.regular_expected_working_time(), Minutes(432));
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn ctrl_del_deletes_the_focused_period_after_confirming() {
    let mut app = app_on_monday();
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
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Right);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 2);
}

#[test]
fn deleting_the_last_period_focuses_the_previous_one() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Char('d'));
    assert_eq!(app.focus(), focus(1, Field::From));
}

#[test]
fn deleting_the_only_period_focuses_from_of_the_empty_period() {
    let mut app = app_with(SAMPLE, date(2026, 11, 6));
    tab_to(&mut app, Field::Notes);
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Char('d'));
    assert!(app.timesheet().days[&date(2026, 11, 6)].periods.is_empty());
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn ctrl_del_on_the_empty_period_of_a_day_without_periods_does_nothing() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    ctrl_key(&mut app, KeyCode::Delete);
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(!screen.contains("─ Delete period ─"));
}

#[test]
fn long_notes_are_cut_off_and_scroll_while_editing() {
    let mut app = app_on_monday();
    // The break (period 1) has no notes yet.
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
    tab_to(&mut app, Field::Break);
    type_text(&mut app, "x");
    assert!(!app.is_editing());
}

#[test]
fn the_cursor_is_shown_in_the_edited_field() {
    let mut app = app_on_monday();
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
fn ctrl_enter_does_not_add_a_period_after_a_new_empty_last_period() {
    // Fri 6 Nov from the sample has one period, 0900–1612.
    let mut app = app_with(SAMPLE, date(2026, 11, 6));
    ctrl_key(&mut app, KeyCode::Enter);
    ctrl_key(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 6)].periods.len(), 2);
    assert_eq!(app.focus(), focus(1, Field::From));
}

#[test]
fn ctrl_enter_does_not_add_a_period_after_an_open_last_period() {
    // Tue 10 Nov from the sample: 0810–1200, then 1230– (open).
    let mut app = app_with(SAMPLE, date(2026, 11, 10));
    ctrl_key(&mut app, KeyCode::Enter);
    assert_eq!(app.focus(), focus(1, Field::From));
    ctrl_key(&mut app, KeyCode::Enter);
    assert_eq!(app.timesheet().days[&date(2026, 11, 10)].periods.len(), 2);
    assert_eq!(app.focus(), focus(1, Field::From));
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("The last period is still open"), "{screen}");
}

#[test]
fn popups_have_a_short_title_and_a_message() {
    let mut app = app_on_monday();
    ctrl_key(&mut app, KeyCode::Delete);
    let screen = rows(&render(&mut app, 120, 24));
    let title = row_index(&screen, "─ Delete period ─");
    let message = row_index(&screen, "Delete period 0845–1230?");
    assert!(message > title, "the message is inside, below the title");
}

#[test]
fn the_consistency_popup_has_a_short_title() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1215");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 's');
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("─ Consistency ─"), "{screen}");
    assert!(
        screen.contains("ma 2 nov: 0845–1230 overlaps 1215–1300"),
        "{screen}"
    );
}

#[test]
fn left_and_right_move_the_cursor_while_editing() {
    let mut app = app_on_monday();
    // The break (period 1) has no notes yet.
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "ac");
    press(&mut app, KeyCode::Left);
    type_text(&mut app, "b");
    press(&mut app, KeyCode::Right);
    type_text(&mut app, "d");
    press(&mut app, KeyCode::Enter);
    assert_eq!(period_at(&app, 1).notes.as_deref(), Some("abcd"));
    assert_eq!(app.focus(), focus(1, Field::Notes));
}

#[test]
fn del_clears_an_input_field() {
    let mut app = app_on_monday();
    press(&mut app, KeyCode::Delete);
    assert_eq!(period_at(&app, 0).from, None);
    tab_to(&mut app, Field::Project);
    press(&mut app, KeyCode::Delete);
    assert_eq!(period_at(&app, 0).project, None);
    tab_to(&mut app, Field::Notes);
    press(&mut app, KeyCode::Delete);
    assert_eq!(period_at(&app, 0).notes, None);
    assert!(!app.is_editing());
}

#[test]
fn del_unchecks_a_checkbox_and_leaves_others_alone() {
    let mut app = app_on_monday();
    // The break (period 1) has Break checked.
    press(&mut app, KeyCode::Down);
    tab_to(&mut app, Field::Free);
    press(&mut app, KeyCode::Delete);
    assert_eq!(
        period_at(&app, 1).period_type,
        tijdje::domain::PeriodType::Break
    );
    press(&mut app, KeyCode::BackTab);
    press(&mut app, KeyCode::Delete);
    assert_eq!(
        period_at(&app, 1).period_type,
        tijdje::domain::PeriodType::Work
    );
}

#[test]
fn del_on_the_empty_period_of_a_day_without_periods_changes_nothing() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    press(&mut app, KeyCode::Delete);
    assert!(app.timesheet().day(date(2026, 11, 11)).periods.is_empty());
    assert!(!app.has_unsaved_changes());
}

/// Starts editing the break's (period 1) Notes with `text`.
fn edit_notes(app: &mut App, text: &str) {
    press(app, KeyCode::Down);
    tab_to(app, Field::Notes);
    type_text(app, text);
}

fn accepted_notes(app: &mut App) -> Option<String> {
    press(app, KeyCode::Enter);
    period_at(app, 1).notes.clone()
}

#[test]
fn ctrl_a_and_ctrl_e_jump_to_the_start_and_end() {
    let mut app = app_on_monday();
    edit_notes(&mut app, "bc");
    ctrl(&mut app, 'a');
    type_text(&mut app, "a");
    ctrl(&mut app, 'e');
    type_text(&mut app, "d");
    assert_eq!(accepted_notes(&mut app).as_deref(), Some("abcd"));
}

#[test]
fn ctrl_w_and_ctrl_backspace_delete_the_previous_word() {
    let mut app = app_on_monday();
    edit_notes(&mut app, "one two  three");
    ctrl(&mut app, 'w');
    ctrl_key(&mut app, KeyCode::Backspace);
    assert_eq!(accepted_notes(&mut app).as_deref(), Some("one "));
}

#[test]
fn ctrl_u_and_ctrl_k_delete_to_the_start_and_end() {
    let mut app = app_on_monday();
    edit_notes(&mut app, "one two three");
    for _ in 0..6 {
        press(&mut app, KeyCode::Left);
    }
    ctrl(&mut app, 'k');
    assert_eq!(accepted_notes(&mut app).as_deref(), Some("one two"));

    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);
    ctrl(&mut app, 'u');
    assert_eq!(accepted_notes(&mut app).as_deref(), Some("two"));
}

#[test]
fn ctrl_left_and_right_move_by_word() {
    let mut app = app_on_monday();
    edit_notes(&mut app, "one two three");
    ctrl_key(&mut app, KeyCode::Left);
    ctrl_key(&mut app, KeyCode::Left);
    type_text(&mut app, "X");
    ctrl_key(&mut app, KeyCode::Right);
    type_text(&mut app, "Y");
    assert_eq!(accepted_notes(&mut app).as_deref(), Some("one XtwoY three"));
}

#[test]
fn ctrl_del_while_editing_neither_deletes_a_character_nor_the_period() {
    let mut app = app_on_monday();
    edit_notes(&mut app, "ab");
    press(&mut app, KeyCode::Left);
    ctrl_key(&mut app, KeyCode::Delete);
    assert_eq!(accepted_notes(&mut app).as_deref(), Some("ab"));
    assert_eq!(app.timesheet().days[&date(2026, 11, 2)].periods.len(), 3);
}

#[test]
fn ctrl_enter_while_editing_the_empty_period_accepts_and_says_it_is_still_open() {
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    type_text(&mut app, "0900");
    ctrl_key(&mut app, KeyCode::Enter);
    assert!(!app.is_editing());
    let periods = &app.timesheet().days[&date(2026, 11, 11)].periods;
    assert_eq!(periods.len(), 1);
    assert_eq!(periods[0].from, TimeOfDay::new(9, 0));
    assert_eq!(app.focus(), focus(0, Field::From));
    let screen = rows(&render(&mut app, 120, 24)).join("\n");
    assert!(screen.contains("The last period is still open"), "{screen}");
}

#[test]
fn ctrl_tab_and_alt_tab_do_not_accept_while_editing() {
    let mut app = app_on_monday();
    type_text(&mut app, "0815");
    ctrl_key(&mut app, KeyCode::Tab);
    alt_key(&mut app, KeyCode::Tab);
    assert!(app.is_editing());
    assert_eq!(app.focus(), focus(0, Field::From));
}
