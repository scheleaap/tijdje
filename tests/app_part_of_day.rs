mod common;

use common::*;
use jiff::civil::{Date, date};
use ratatui::crossterm::event::KeyCode;
use tijdje::{
    app::{App, Field},
    domain::{PartOfDay, Period, TimeOfDay},
};

const DAY: Date = date(2026, 11, 12);

const FIRST_HALF_OF_VACATION: &str = r#"{ from: null, to: null, part_of_day: first_half, project: null, notes: null, type: vacation }"#;
const FULL_HOLIDAY: &str =
    r#"{ from: null, to: null, part_of_day: full_day, project: null, notes: null, type: holiday }"#;
const ACME_0900_1000: &str =
    r#"{ from: "09:00", to: "10:00", project: acme, notes: null, type: work }"#;
const ACME_1100_1200: &str =
    r#"{ from: "11:00", to: "12:00", project: acme, notes: null, type: work }"#;
const ACME_1300_1400: &str =
    r#"{ from: "13:00", to: "14:00", project: acme, notes: null, type: work }"#;
const BETA_1400_1500: &str =
    r#"{ from: "14:00", to: "15:00", project: beta, notes: null, type: work }"#;

/// Thu 12 Nov, today, with `periods` in YAML flow style.
fn app_on(periods: &[&str]) -> App {
    app_with(&one_day_yaml("2026-11-12", periods), DAY)
}

fn screen(app: &mut App) -> Vec<String> {
    rows(&render(app, 120, 24))
}

fn part_of_day(app: &App, index: usize) -> Option<PartOfDay> {
    period_at(app, index).part_of_day()
}

// Showing the part of day

#[test]
fn the_label_replaces_from_and_to_and_duration_shows_its_duration() {
    // Given
    let mut app = app_on(&[FIRST_HALF_OF_VACATION]);

    // When
    let rows = screen(&mut app);

    // Then: the label fills From, the gap and To; then the duration.
    let row = row_containing(&rows, "1st half");
    assert!(row.contains("1st half  3h36m"), "{row:?}");
}

#[test]
fn focus_starts_on_the_part_of_day_field_of_a_part_of_day_period() {
    assert_eq!(
        app_on(&[FIRST_HALF_OF_VACATION]).focus(),
        focus(0, Field::PartOfDay)
    );
}

#[test]
fn an_implied_break_skips_a_part_of_day_period_in_between() {
    // Given
    let mut app = app_on(&[ACME_0900_1000, FULL_HOLIDAY, ACME_1100_1200]);

    // When
    let rows = screen(&mut app);

    // Then
    let row = row_containing(&rows, "1000 1100");
    assert!(right(row).starts_with("│☕"), "{row:?}");
}

// Keys on the part-of-day field

#[test]
fn enter_and_typing_do_nothing_on_the_part_of_day_field() {
    // Given
    let mut app = app_on(&[FIRST_HALF_OF_VACATION]);

    // When
    press(&mut app, KeyCode::Enter);
    type_text(&mut app, "1x");

    // Then
    assert!(!app.is_editing());
    assert_eq!(part_of_day(&app, 0), Some(PartOfDay::FirstHalf));
    assert!(!app.has_unsaved_changes());
}

#[test]
fn del_switches_the_part_of_day_back_to_from_to() {
    // Given
    let mut app = app_on(&[FIRST_HALF_OF_VACATION]);

    // When
    press(&mut app, KeyCode::Delete);

    // Then
    let period = period_at(&app, 0);
    assert_eq!(
        (period.part_of_day(), period.from(), period.to()),
        (None, None, None)
    );
    assert_eq!(app.focus(), focus(0, Field::From));
    assert!(!screen(&mut app).join("\n").contains("1st half"));
}

#[test]
fn ctrl_c_and_ctrl_v_do_nothing_on_the_part_of_day_field() {
    // Given
    let clipboard = FakeClipboard::with("1300");
    let mut app = app_on(&[FIRST_HALF_OF_VACATION]);
    app.set_clipboard(Box::new(clipboard.clone()));

    // When
    ctrl(&mut app, 'v');
    ctrl(&mut app, 'c');

    // Then
    assert_eq!(part_of_day(&app, 0), Some(PartOfDay::FirstHalf));
    assert_eq!(clipboard.text().as_deref(), Some("1300"));
}

// Alt+0..3

#[test]
fn alt_1_2_3_switch_to_full_day_1st_half_and_2nd_half() {
    for (key, part, label) in [
        ('1', PartOfDay::FullDay, "Full day  7h12m"),
        ('2', PartOfDay::FirstHalf, "1st half  3h36m"),
        ('3', PartOfDay::SecondHalf, "2nd half  3h36m"),
    ] {
        // Given
        let mut app = app_on(&[ACME_1300_1400]);
        tab_to(&mut app, Field::Project);

        // When
        alt_key(&mut app, KeyCode::Char(key));

        // Then
        let period = period_at(&app, 0);
        assert_eq!(
            (period.part_of_day(), period.from(), period.to()),
            (Some(part), None, None),
            "Alt+{key}"
        );
        assert_eq!(app.focus(), focus(0, Field::PartOfDay), "Alt+{key}");
        let rows = screen(&mut app);
        assert!(row_containing(&rows, "acme").contains(label), "Alt+{key}");
    }
}

#[test]
fn alt_0_switches_a_part_of_day_period_to_from_to() {
    // Given
    let mut app = app_on(&[FIRST_HALF_OF_VACATION]);
    tab_to(&mut app, Field::Notes);

    // When
    alt_key(&mut app, KeyCode::Char('0'));

    // Then
    assert_eq!(part_of_day(&app, 0), None);
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn alt_0_does_nothing_on_a_from_to_period() {
    // Given
    let mut app = app_on(&[ACME_1300_1400]);
    tab_to(&mut app, Field::Project);
    let before = period_at(&app, 0).clone();

    // When
    alt_key(&mut app, KeyCode::Char('0'));

    // Then
    assert_eq!(period_at(&app, 0), &before);
    assert_eq!(app.focus(), focus(0, Field::Project));
    assert!(!app.has_unsaved_changes());
}

#[test]
fn alt_digits_accept_the_active_input_first() {
    // Given
    let mut app = app_on(&[ACME_1300_1400]);
    tab_to(&mut app, Field::Notes);
    type_text(&mut app, "Dentist");

    // When
    alt_key(&mut app, KeyCode::Char('3'));

    // Then
    assert!(!app.is_editing());
    assert_eq!(period_at(&app, 0).notes.as_deref(), Some("Dentist"));
    assert_eq!(part_of_day(&app, 0), Some(PartOfDay::SecondHalf));
}

#[test]
fn alt_digits_do_nothing_when_the_active_input_is_invalid() {
    // Given
    let mut app = app_on(&[ACME_1300_1400]);
    type_text(&mut app, "99");

    // When
    alt_key(&mut app, KeyCode::Char('1'));

    // Then
    assert!(app.is_editing());
    assert_eq!(part_of_day(&app, 0), None);
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn alt_digits_do_nothing_on_the_day_note() {
    // Given
    let mut app = app_on(&[ACME_1300_1400]);
    focus_day_note(&mut app);

    // When
    for key in ['0', '1', '2', '3'] {
        alt_key(&mut app, KeyCode::Char(key));
    }

    // Then
    assert_eq!(app.focus(), DAY_NOTE);
    assert_eq!(part_of_day(&app, 0), None);
    assert!(!app.has_unsaved_changes());
}

#[test]
fn a_part_of_day_switch_can_be_undone() {
    // Given
    let mut app = app_on(&[ACME_1300_1400]);
    alt_key(&mut app, KeyCode::Char('1'));

    // When
    ctrl(&mut app, 'z');

    // Then
    assert_eq!(part_of_day(&app, 0), None);
    assert_eq!(period_at(&app, 0).from(), TimeOfDay::new(13, 0));
    assert_eq!(app.focus(), focus(0, Field::From));
}

// Field navigation

#[test]
fn up_and_down_treat_the_part_of_day_field_as_above_from_and_to() {
    // Given: To of 1300–1400 has focus.
    let mut app = app_on(&[FIRST_HALF_OF_VACATION, ACME_1300_1400]);
    press(&mut app, KeyCode::Down);
    press(&mut app, KeyCode::Right);
    let start = app.focus();

    // When
    press(&mut app, KeyCode::Up);
    let up = app.focus();
    press(&mut app, KeyCode::Down);

    // Then
    assert_eq!(start, focus(1, Field::To));
    assert_eq!(up, focus(0, Field::PartOfDay));
    assert_eq!(app.focus(), focus(1, Field::From));
}

#[test]
fn up_from_the_day_note_lands_on_a_last_part_of_day_field() {
    // Given
    let mut app = app_on(&[ACME_1300_1400, FIRST_HALF_OF_VACATION]);
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Up);

    // Then
    assert_eq!(app.focus(), focus(1, Field::PartOfDay));
}

#[test]
fn left_and_right_treat_the_part_of_day_field_as_one_field() {
    // Given
    let mut app = app_on(&[FIRST_HALF_OF_VACATION]);

    // When
    press(&mut app, KeyCode::Right);
    let right = app.focus();
    press(&mut app, KeyCode::Left);
    press(&mut app, KeyCode::Left);

    // Then
    assert_eq!(right, focus(0, Field::Project));
    assert_eq!(app.focus(), focus(0, Field::PartOfDay));
}

#[test]
fn tab_walks_through_the_part_of_day_field() {
    // Given: the day note comes last, so Tab wraps around to the first period.
    let mut app = app_on(&[FIRST_HALF_OF_VACATION]);
    focus_day_note(&mut app);

    // When
    press(&mut app, KeyCode::Tab);
    let after_day_note = app.focus();
    press(&mut app, KeyCode::Tab);
    let after_part_of_day = app.focus();
    press(&mut app, KeyCode::BackTab);

    // Then
    assert_eq!(after_day_note, focus(0, Field::PartOfDay));
    assert_eq!(after_part_of_day, focus(0, Field::Project));
    assert_eq!(app.focus(), focus(0, Field::PartOfDay));
}

#[test]
fn ctrl_enter_after_a_part_of_day_last_period_adds_a_period_with_an_empty_from() {
    // Given
    let mut app = app_on(&[ACME_1300_1400, FIRST_HALF_OF_VACATION]);
    press(&mut app, KeyCode::Down);

    // When
    ctrl_key(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(app.timesheet().day(DAY).periods.len(), 3);
    assert_eq!(period_at(&app, 2), &Period::default());
    assert_eq!(app.focus(), focus(2, Field::From));
}

// Period types, sorting and saving

#[test]
fn checking_vacation_on_an_empty_day_shows_a_full_day() {
    // Given: Wed 11 Nov has no periods.
    let mut app = app_with(SAMPLE, date(2026, 11, 11));

    // When
    alt_key(&mut app, KeyCode::Char('v'));

    // Then
    let rows = screen(&mut app);
    assert!(row_containing(&rows, "Full day").contains("Full day  7h12m"));
    assert_eq!(app.focus(), focus(0, Field::PartOfDay));
}

#[test]
fn unchecking_vacation_removes_the_full_day_again() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Char('v'));

    // When
    alt_key(&mut app, KeyCode::Char('v'));

    // Then
    assert!(app.timesheet().day(date(2026, 11, 11)).is_empty());
    assert_eq!(app.focus(), focus(0, Field::From));
}

#[test]
fn leaving_the_day_sorts_part_of_day_periods_around_the_from_to_periods() {
    // Given: 1400–1500 becomes a 2nd half, before 1300–1400 on screen.
    let mut app = app_on(&[BETA_1400_1500, ACME_1300_1400, FIRST_HALF_OF_VACATION]);
    alt_key(&mut app, KeyCode::Char('3'));

    // When
    ctrl_key(&mut app, KeyCode::Down);
    ctrl_key(&mut app, KeyCode::Up);

    // Then
    let parts: Vec<Option<PartOfDay>> = (0..3).map(|i| part_of_day(&app, i)).collect();
    assert_eq!(
        parts,
        vec![
            Some(PartOfDay::FirstHalf),
            None,
            Some(PartOfDay::SecondHalf)
        ]
    );
}

#[test]
fn a_failed_check_names_the_part_of_day_by_its_label() {
    // Given: 1300–1400 becomes a second 1st half.
    let mut app = app_on(&[FIRST_HALF_OF_VACATION, ACME_1300_1400]);
    press(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Char('2'));

    // When
    ctrl_key(&mut app, KeyCode::Down);

    // Then
    assert_eq!(app.selected_day(), DAY);
    assert!(
        screen(&mut app)
            .join("\n")
            .contains("More than one 1st half")
    );
}

#[test]
fn a_part_of_day_is_saved() {
    // Given
    let (mut app, fs) = app_with_fs(&one_day_yaml("2026-11-12", &[BETA_1400_1500]), DAY);
    alt_key(&mut app, KeyCode::Char('3'));

    // When
    ctrl(&mut app, 's');

    // Then
    let yaml = fs.get(TIMESHEET).unwrap();
    assert!(
        yaml.contains(
            "      - from: null\n        to: null\n        part_of_day: second_half\n        project: \"beta\"\n"
        ),
        "{yaml}"
    );
}

#[test]
fn help_lists_the_part_of_day_keys() {
    // Given
    let mut app = app_on(&[ACME_1300_1400]);

    // When
    press(&mut app, KeyCode::F(1));

    // Then
    let rows = screen(&mut app);
    row_containing(&rows, "Alt+0/1/2/3");
    row_containing(&rows, "Ctrl+S/Ctrl+Q");
}
