mod common;

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::{
    app::{App, Field, FieldFocus},
    domain::{PeriodType, TimeOfDay},
};

/// Mon 2 Nov from the sample: 0845–1230 work, 1230–1300 break, 1300–1757 work.
fn monday_with_first_period_focused() -> App {
    app_with(SAMPLE, date(2026, 11, 2))
}

fn first_period(app: &App) -> &tijdje::domain::Period {
    &app.timesheet().days[&app.selected_day()].periods[0]
}

fn screen(app: &mut App) -> Vec<String> {
    rows(&render(app, 120, 24))
}

fn tab_to(app: &mut App, field: Field) {
    while app.focus().field != field {
        press(app, KeyCode::Tab);
    }
}

#[test]
fn enter_toggles_the_focused_checkbox() {
    let mut app = monday_with_first_period_focused();
    tab_to(&mut app, Field::Break);
    press(&mut app, KeyCode::Enter);
    assert_eq!(first_period(&app).period_type, PeriodType::Break);
    // 1300–1757 is the only work period left.
    assert!(row_containing(&screen(&mut app), "ma  2 nov").contains("4h57m"));
    press(&mut app, KeyCode::Enter);
    assert_eq!(first_period(&app).period_type, PeriodType::Work);
    assert!(app.has_unsaved_changes());
}

#[test]
fn alt_letters_toggle_period_types_from_any_field() {
    let mut app = monday_with_first_period_focused();
    for (key, period_type) in [
        ('b', PeriodType::Break),
        ('f', PeriodType::Free),
        ('v', PeriodType::Vacation),
        ('s', PeriodType::Sick),
        ('h', PeriodType::Holiday),
    ] {
        alt_key(&mut app, KeyCode::Char(key));
        assert_eq!(first_period(&app).period_type, period_type, "Alt+{key}");
    }
    alt_key(&mut app, KeyCode::Char('h'));
    assert_eq!(first_period(&app).period_type, PeriodType::Work);
}

#[test]
fn checking_an_absence_updates_the_figures_and_indicator() {
    let mut app = monday_with_first_period_focused();
    alt_key(&mut app, KeyCode::Char('v'));
    let rows = screen(&mut app);
    let monday = row_containing(&rows, "ma  2 nov");
    assert!(
        monday.contains("🏖️") && monday.contains("4h57m"),
        "{monday:?}"
    );
    // 7h12m − 3h45m vacation
    assert!(
        rows.join("\n")
            .contains("Actual expected: 3h27m (regular 7h12m − 3h45m absence)")
    );
    let row = row_containing(&rows, "0845 1230");
    assert_eq!(row.matches('✓').count(), 1, "{row:?}");
}

#[test]
fn an_absence_on_an_empty_day_fills_a_full_day_and_unchecking_empties_it() {
    // Wed 11 Nov has no periods: the absence goes on the empty period it shows.
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Char('v'));
    assert_eq!(first_period(&app).from, TimeOfDay::new(9, 0));
    assert_eq!(first_period(&app).to, TimeOfDay::new(16, 12));
    let rows = screen(&mut app);
    let row = row_containing(&rows, "wo 11 nov");
    assert!(
        row.split_whitespace().any(|c| c == "0h00m"),
        "day balance 0h00m: {row:?}"
    );

    alt_key(&mut app, KeyCode::Char('v'));
    assert!(
        app.timesheet().day(date(2026, 11, 11)).is_empty(),
        "an empty period again, which stays until the day is left and isn't saved"
    );
}

#[test]
fn alt_letters_accept_the_active_input_first() {
    let mut app = monday_with_first_period_focused();
    type_text(&mut app, "0815");
    alt_key(&mut app, KeyCode::Char('b'));
    assert!(!app.is_editing());
    assert_eq!(first_period(&app).from, TimeOfDay::new(8, 15));
    assert_eq!(first_period(&app).period_type, PeriodType::Break);
}

#[test]
fn alt_letters_do_nothing_when_the_active_input_is_invalid() {
    let mut app = monday_with_first_period_focused();
    type_text(&mut app, "99");
    alt_key(&mut app, KeyCode::Char('b'));
    alt_key(&mut app, KeyCode::Char('v'));
    assert!(app.is_editing());
    assert_eq!(first_period(&app).period_type, PeriodType::Work);
    assert_eq!(app.focus().field, Field::From);
}

#[test]
fn alt_p_and_alt_n_are_removed() {
    let mut app = monday_with_first_period_focused();
    type_text(&mut app, "0815");
    alt_key(&mut app, KeyCode::Char('p'));
    alt_key(&mut app, KeyCode::Char('n'));
    assert!(app.is_editing(), "ignored while editing");
    assert_eq!(
        app.focus(),
        FieldFocus {
            period: 0,
            field: Field::From
        }
    );
    press(&mut app, KeyCode::Esc);
    alt_key(&mut app, KeyCode::Char('p'));
    alt_key(&mut app, KeyCode::Char('n'));
    assert_eq!(app.focus().field, Field::From);
}

#[test]
fn del_on_a_checked_absence_unchecks_it_like_enter() {
    // Wed 11 Nov has no periods: checking Vacation fills in a full day.
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Char('v'));
    assert_eq!(first_period(&app).period_type, PeriodType::Vacation);
    while app.focus().field != Field::Vacation {
        press(&mut app, KeyCode::Tab);
    }
    press(&mut app, KeyCode::Delete);
    assert!(app.timesheet().day(date(2026, 11, 11)).is_empty());
}

#[test]
fn alt_letters_work_right_after_selecting_a_day() {
    let mut app = app_with(SAMPLE, date(2026, 11, 2));
    alt_key(&mut app, KeyCode::Char('b'));
    assert_eq!(first_period(&app).period_type, PeriodType::Break);
}

#[test]
fn alt_letters_on_a_day_without_periods_make_the_empty_period_real() {
    // Wed 11 Nov has no periods in the sample.
    let mut app = app_with(SAMPLE, date(2026, 11, 11));
    alt_key(&mut app, KeyCode::Char('b'));
    let periods = &app.timesheet().days[&date(2026, 11, 11)].periods;
    assert_eq!(periods.len(), 1);
    assert_eq!(periods[0].period_type, PeriodType::Break);
}

#[test]
fn space_on_a_text_field_types_a_space() {
    let mut app = monday_with_first_period_focused();
    tab_to(&mut app, Field::Notes);
    press(&mut app, KeyCode::Char(' '));
    assert!(app.is_editing());
}

#[test]
fn period_types_are_saved() {
    // The break (1230–1300) has no project, so it may become a sick period.
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Char('s'));
    ctrl(&mut app, 's');
    let yaml = fs.get(TIMESHEET).unwrap();
    assert!(
        yaml.contains(
            "      - from: \"12:30\"\n        to: \"13:00\"\n        project: null\n        notes: null\n        type: sick\n"
        ),
        "{yaml}"
    );
}

#[test]
fn space_does_not_toggle_a_checkbox() {
    let mut app = monday_with_first_period_focused();
    tab_to(&mut app, Field::Break);
    press(&mut app, KeyCode::Char(' '));
    assert_eq!(first_period(&app).period_type, PeriodType::Work);
    assert!(!app.is_editing());
}

/// The break (1230–1300) of Mon 2 Nov as a free period.
fn monday_with_a_free_period() -> App {
    let mut app = monday_with_first_period_focused();
    press(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Char('f'));
    app
}

#[test]
fn the_free_checkbox_header_is_between_break_and_vacation() {
    // Given
    let mut app = monday_with_first_period_focused();

    // When
    let rows = screen(&mut app);

    // Then
    let header = row_containing(&rows, "Project");
    let positions: Vec<usize> = ["☕", "🆓", "🏖️"]
        .iter()
        .map(|emoji| header.find(emoji).expect(emoji))
        .collect();
    assert!(positions.is_sorted(), "{header:?}");
}

#[test]
fn enter_on_the_free_checkbox_toggles_free() {
    // Given
    let mut app = monday_with_first_period_focused();
    tab_to(&mut app, Field::Free);

    // When
    press(&mut app, KeyCode::Enter);

    // Then
    assert_eq!(first_period(&app).period_type, PeriodType::Free);
    let rows = screen(&mut app);
    let row = row_containing(&rows, "0845 1230");
    assert_eq!(row.matches('✓').count(), 1, "{row:?}");
}

#[test]
fn a_free_period_shows_its_indicator_left_of_from() {
    // Given
    let mut app = monday_with_a_free_period();

    // When
    let rows = screen(&mut app);

    // Then
    let row = row_containing(&rows, "1230 1300");
    assert!(right(row).starts_with("│🆓"), "{row:?}");
}

#[test]
fn a_free_period_is_not_worked_time() {
    // Given: 0845–1230 work becomes free, leaving 1300–1757 (4h57m) worked.
    let mut app = monday_with_first_period_focused();

    // When
    alt_key(&mut app, KeyCode::Char('f'));

    // Then
    let rows = screen(&mut app);
    let monday = left(row_containing(&rows, "ma  2 nov"));
    assert!(monday.contains("4h57m"), "{monday:?}");
}

#[test]
fn a_free_period_has_no_absence_indicator() {
    // Given
    let mut app = monday_with_a_free_period();

    // When
    let rows = screen(&mut app);

    // Then
    let monday = left(row_containing(&rows, "ma  2 nov"));
    assert!(
        !["🏖️", "🤒", "🎄"].iter().any(|e| monday.contains(e)),
        "{monday:?}"
    );
}

#[test]
fn a_free_period_does_not_reduce_the_actual_expected_working_time() {
    // Given
    let mut app = monday_with_a_free_period();

    // When
    let screen = screen(&mut app).join("\n");

    // Then
    assert!(screen.contains("Actual expected: 7h12m"), "{screen}");
    assert!(!screen.contains("absence)"), "{screen}");
}

#[test]
fn a_free_period_on_an_empty_day_does_not_fill_times() {
    // Given: Wed 11 Nov has no periods.
    let mut app = app_with(SAMPLE, date(2026, 11, 11));

    // When
    alt_key(&mut app, KeyCode::Char('f'));

    // Then
    assert_eq!(first_period(&app).period_type, PeriodType::Free);
    assert_eq!(
        (first_period(&app).from, first_period(&app).to),
        (None, None)
    );
}

#[test]
fn alt_f_accepts_the_active_input_first() {
    // Given
    let mut app = monday_with_first_period_focused();
    type_text(&mut app, "0815");

    // When
    alt_key(&mut app, KeyCode::Char('f'));

    // Then
    assert!(!app.is_editing());
    assert_eq!(first_period(&app).from, TimeOfDay::new(8, 15));
    assert_eq!(first_period(&app).period_type, PeriodType::Free);
}

#[test]
fn free_periods_are_saved() {
    // Given
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    press(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Char('f'));

    // When
    ctrl(&mut app, 's');

    // Then
    let yaml = fs.get(TIMESHEET).unwrap();
    assert!(
        yaml.contains(
            "      - from: \"12:30\"\n        to: \"13:00\"\n        project: null\n        notes: null\n        type: free\n"
        ),
        "{yaml}"
    );
}
