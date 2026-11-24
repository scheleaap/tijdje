mod common;

use std::path::{Path, PathBuf};

use common::*;
use jiff::civil::date;
use ratatui::crossterm::event::KeyCode;
use tijdje::{
    app::{App, Field, Outcome},
    domain::TimeOfDay,
    store,
};

/// Changes Mon 2 Nov's first From to 0815 on the in-memory sample file.
fn app_with_a_change() -> (App, InMemoryFileSystem) {
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    type_text(&mut app, "0815");
    press(&mut app, KeyCode::Enter);
    (app, fs)
}

fn screen(app: &mut App) -> String {
    rows(&render(app, 120, 24)).join("\n")
}

fn reload(fs: &InMemoryFileSystem) -> tijdje::domain::Timesheet {
    store::load(fs, Path::new(TIMESHEET)).unwrap().0
}

#[test]
fn ctrl_s_saves_the_timesheet() {
    let (mut app, fs) = app_with_a_change();
    assert!(app.has_unsaved_changes());
    assert_eq!(ctrl(&mut app, 's'), Outcome::Continue);
    assert!(!app.has_unsaved_changes());
    let saved = reload(&fs);
    assert_eq!(
        saved.days[&date(2026, 11, 2)].periods[0].from(),
        TimeOfDay::new(8, 15)
    );
    assert_eq!(&saved, app.timesheet());
}

#[test]
fn saved_yaml_follows_the_documented_schema() {
    let (mut app, fs) = app_with_a_change();
    ctrl(&mut app, 's');
    let yaml = fs.get(TIMESHEET).unwrap();
    let monday = "  \"2026-11-02\":
    regular_expected_working_time_in_minutes: 432
    notes: null
    periods:
      - from: \"08:15\"
        to: \"12:30\"
        part_of_day: null
        project: \"internal\"
        notes: \"Onboarding\"
        type: work
      - from: \"12:30\"
        to: \"13:00\"
        part_of_day: null
        project: null
        notes: null
        type: break
";
    assert!(yaml.starts_with("schema_version: 1\ndays:\n"), "{yaml}");
    assert!(yaml.contains(monday), "{yaml}");
    assert!(
        yaml.contains("        to: null\n"),
        "open period's To is null"
    );
}

#[test]
fn text_with_quotes_and_special_characters_survives_saving() {
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    // The break (period 1) has no notes yet.
    press(&mut app, KeyCode::Down);
    while app.focus().field() != Some(Field::Notes) {
        press(&mut app, KeyCode::Right);
    }
    let notes = r#"say "hi" \ # : - [x] {y} 'z' ü"#;
    type_text(&mut app, notes);
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 's');
    let saved = reload(&fs);
    assert_eq!(
        saved.days[&date(2026, 11, 2)].periods[1].notes.as_deref(),
        Some(notes)
    );
}

#[test]
fn days_without_periods_are_not_written() {
    let (mut app, fs) = sample_on(date(2026, 11, 6));
    ctrl_key(&mut app, KeyCode::Delete);
    press(&mut app, KeyCode::Char('d'));
    ctrl(&mut app, 's');
    assert!(!fs.get(TIMESHEET).unwrap().contains("2026-11-06"));
}

#[test]
fn a_new_day_stores_the_regular_expected_working_time() {
    let (mut app, fs) = sample_on(date(2026, 11, 11));
    type_text(&mut app, "0900");
    press(&mut app, KeyCode::Tab);
    type_text(&mut app, "1200");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 's');
    assert!(
        fs.get(TIMESHEET)
            .unwrap()
            .contains("  \"2026-11-11\":\n    regular_expected_working_time_in_minutes: 432\n")
    );
}

#[test]
fn existing_regular_expected_working_time_is_kept() {
    let yaml = "days:\n  \"2026-11-02\":\n    regular_expected_working_time_in_minutes: 240\n    periods:\n      - { from: \"09:00\", to: \"13:00\", project: null, notes: null, type: work }\n";
    let (mut app, fs) = app_with_fs(yaml, date(2026, 11, 2));
    type_text(&mut app, "0800");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 's');
    assert!(
        fs.get(TIMESHEET)
            .unwrap()
            .contains("regular_expected_working_time_in_minutes: 240")
    );
}

#[test]
fn the_previous_file_is_kept_as_a_backup() {
    let (mut app, fs) = app_with_a_change();
    ctrl(&mut app, 's');
    assert_eq!(fs.get("timesheet.yaml.bak").as_deref(), Some(SAMPLE));
    assert_eq!(fs.get("timesheet.yaml.tmp"), None);
}

#[test]
fn the_backup_keeps_the_file_from_before_the_session() {
    let (mut app, fs) = app_with_a_change();
    ctrl(&mut app, 's');
    press(&mut app, KeyCode::Down);
    type_text(&mut app, "1300");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 's');
    assert_eq!(fs.get("timesheet.yaml.bak").as_deref(), Some(SAMPLE));
}

#[test]
fn accepting_an_unchanged_value_is_not_an_unsaved_change() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    type_text(&mut app, "0845");
    press(&mut app, KeyCode::Enter);
    assert!(!app.has_unsaved_changes());
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Quit);
}

#[test]
fn saving_a_new_file_creates_it_without_a_backup() {
    let fs = InMemoryFileSystem::default();
    let mut app = App::open(
        Box::new(fs.clone()),
        PathBuf::from(TIMESHEET),
        Box::new(FakeClock::new(date(2026, 11, 2))),
    )
    .unwrap();
    type_text(&mut app, "0900");
    press(&mut app, KeyCode::Enter);
    ctrl(&mut app, 's');
    assert!(fs.get(TIMESHEET).unwrap().contains("\"09:00\""));
    assert_eq!(fs.get("timesheet.yaml.bak"), None);
}

#[test]
fn saving_after_saving_needs_no_confirmation() {
    let (mut app, _fs) = app_with_a_change();
    ctrl(&mut app, 's');
    ctrl(&mut app, 's');
    assert!(!screen(&mut app).contains("─ File changed ─"));
}

#[test]
fn a_file_changed_on_disk_asks_before_overwriting() {
    let (mut app, fs) = app_with_a_change();
    fs.set(TIMESHEET, "days: {}\n");
    ctrl(&mut app, 's');
    let text = screen(&mut app);
    assert!(text.contains("─ File changed ─") && text.contains(" Overwrite "));
    press(&mut app, KeyCode::Char('c'));
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some("days: {}\n"));
    assert!(app.has_unsaved_changes());

    ctrl(&mut app, 's');
    press(&mut app, KeyCode::Char('o'));
    assert_eq!(
        reload(&fs).days[&date(2026, 11, 2)].periods[0].from(),
        TimeOfDay::new(8, 15)
    );
    assert!(!app.has_unsaved_changes());
}

#[test]
fn ctrl_q_without_unsaved_changes_quits() {
    let (mut app, _fs) = sample_on(date(2026, 11, 2));
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Quit);
}

#[test]
fn ctrl_q_with_unsaved_changes_asks_to_save() {
    let (mut app, _fs) = app_with_a_change();
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Continue);
    let text = screen(&mut app);
    assert!(text.contains("Save before quitting?"));
    assert!(text.contains(" Save ") && text.contains(" Don't save ") && text.contains(" Cancel "));
}

#[test]
fn save_before_quitting_saves_and_quits() {
    let (mut app, fs) = app_with_a_change();
    ctrl(&mut app, 'q');
    assert_eq!(press(&mut app, KeyCode::Char('s')), Outcome::Quit);
    assert_eq!(
        reload(&fs).days[&date(2026, 11, 2)].periods[0].from(),
        TimeOfDay::new(8, 15)
    );
}

#[test]
fn dont_save_quits_without_saving() {
    let (mut app, fs) = app_with_a_change();
    ctrl(&mut app, 'q');
    press(&mut app, KeyCode::Right);
    assert_eq!(press(&mut app, KeyCode::Enter), Outcome::Quit);
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some(SAMPLE));
}

#[test]
fn cancel_stays_in_the_application() {
    let (mut app, fs) = app_with_a_change();
    ctrl(&mut app, 'q');
    assert_eq!(press(&mut app, KeyCode::Esc), Outcome::Continue);
    assert!(!screen(&mut app).contains("Save before quitting?"));
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some(SAMPLE));
}

#[test]
fn quitting_with_a_file_changed_on_disk_asks_and_then_quits() {
    let (mut app, fs) = app_with_a_change();
    fs.set(TIMESHEET, "days: {}\n");
    ctrl(&mut app, 'q');
    assert_eq!(press(&mut app, KeyCode::Char('s')), Outcome::Continue);
    assert!(screen(&mut app).contains("─ File changed ─"));
    assert_eq!(press(&mut app, KeyCode::Char('o')), Outcome::Quit);
    assert!(fs.get(TIMESHEET).unwrap().contains("\"08:15\""));
}

#[test]
fn canceling_the_overwrite_while_quitting_stays_in_the_application() {
    let (mut app, fs) = app_with_a_change();
    fs.set(TIMESHEET, "days: {}\n");
    ctrl(&mut app, 'q');
    press(&mut app, KeyCode::Char('s'));
    assert_eq!(press(&mut app, KeyCode::Esc), Outcome::Continue);
    // A later Ctrl+S doesn't quit.
    ctrl(&mut app, 's');
    assert_eq!(press(&mut app, KeyCode::Char('o')), Outcome::Continue);
}

#[test]
fn ctrl_s_and_ctrl_q_are_ignored_while_editing() {
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    type_text(&mut app, "07");
    ctrl(&mut app, 's');
    assert_eq!(ctrl(&mut app, 'q'), Outcome::Continue);
    assert!(app.is_editing());
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some(SAMPLE));
    assert!(!screen(&mut app).contains("Save before quitting?"));
}

#[test]
fn an_unparsable_file_cannot_be_opened() {
    let fs = InMemoryFileSystem::with(TIMESHEET, "days: [unclosed");
    let result = App::open(
        Box::new(fs.clone()),
        PathBuf::from(TIMESHEET),
        Box::new(FakeClock::new(date(2026, 11, 2))),
    );
    assert!(result.is_err());
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some("days: [unclosed"));
}

#[test]
fn a_file_with_another_schema_version_cannot_be_opened() {
    let yaml = "schema_version: 2\ndays: {}\n";
    let fs = InMemoryFileSystem::with(TIMESHEET, yaml);
    let result = App::open(
        Box::new(fs.clone()),
        PathBuf::from(TIMESHEET),
        Box::new(FakeClock::new(date(2026, 11, 2))),
    );
    let Err(err) = result else {
        panic!("a file with schema version 2 was opened");
    };
    assert!(err.to_string().contains("schema version 2"), "{err}");
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some(yaml));
}
