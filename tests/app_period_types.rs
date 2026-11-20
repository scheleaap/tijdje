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
        app.timesheet().day(date(2026, 11, 11)).periods.is_empty(),
        "back to the empty period, which isn't a real period"
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
    alt_key(&mut app, KeyCode::Char('p'));
    assert!(app.is_editing());
    assert_eq!(first_period(&app).period_type, PeriodType::Work);
    assert_eq!(app.focus().field, Field::From);
}

#[test]
fn alt_p_and_alt_n_go_to_project_and_notes() {
    let mut app = monday_with_first_period_focused();
    type_text(&mut app, "0815");
    alt_key(&mut app, KeyCode::Char('p'));
    assert_eq!(
        app.focus(),
        FieldFocus {
            period: 0,
            field: Field::Project
        }
    );
    assert_eq!(first_period(&app).from, TimeOfDay::new(8, 15));
    alt_key(&mut app, KeyCode::Char('n'));
    assert_eq!(app.focus().field, Field::Notes);
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
