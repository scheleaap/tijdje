mod common;

use std::collections::BTreeMap;

use common::period;
use jiff::civil::{Date, date};
use tijdje::domain::{Day, Period, PeriodType, Timesheet};

const TODAY: Date = date(2026, 11, 13);

fn day_with_only_a_note() -> Day {
    Day {
        day_note: Some("Team day".to_owned()),
        ..Day::default()
    }
}

fn day_with_a_period() -> Day {
    Day {
        periods: vec![period(900, Some(1200), PeriodType::Work)],
        ..Day::default()
    }
}

#[test]
fn setting_the_day_note_keeps_its_text() {
    // Given
    let mut day = Day::default();

    // When
    day.set_day_note(" Team day ");

    // Then
    assert_eq!(day.day_note.as_deref(), Some(" Team day "));
}

#[test]
fn a_day_note_of_only_spaces_is_no_day_note() {
    // Given
    let mut day = day_with_only_a_note();

    // When
    day.set_day_note("   ");

    // Then
    assert_eq!(day.day_note, None);
}

#[test]
fn an_empty_day_note_is_no_day_note() {
    // Given
    let mut day = day_with_only_a_note();

    // When
    day.set_day_note("");

    // Then
    assert_eq!(day.day_note, None);
}

#[test]
fn a_day_with_only_a_note_is_an_empty_day() {
    assert!(day_with_only_a_note().is_empty());
}

#[test]
fn a_day_with_only_a_note_has_content() {
    assert!(day_with_only_a_note().has_content());
}

#[test]
fn a_day_with_a_period_has_content() {
    assert!(day_with_a_period().has_content());
}

#[test]
fn a_day_without_a_note_and_with_only_empty_periods_has_no_content() {
    // Given
    let day = Day {
        periods: vec![Period::default()],
        ..Day::default()
    };

    // When
    let has_content = day.has_content();

    // Then
    assert!(!has_content);
}

#[test]
fn a_day_with_only_a_note_is_not_a_recorded_day() {
    // Given
    let timesheet = Timesheet {
        days: BTreeMap::from([
            (date(2026, 11, 2), day_with_only_a_note()),
            (date(2026, 11, 4), day_with_a_period()),
        ]),
    };

    // When
    let first = timesheet.first_recorded_day();

    // Then
    assert_eq!(first, Some(date(2026, 11, 4)));
    assert_eq!(timesheet.worked_time_on(date(2026, 11, 2)), None);
}

#[test]
fn a_past_day_with_only_a_note_has_a_day_warning() {
    // Given
    let timesheet = Timesheet {
        days: BTreeMap::from([
            (date(2026, 11, 2), day_with_a_period()),
            (date(2026, 11, 3), day_with_only_a_note()),
        ]),
    };

    // When
    let warns = timesheet.has_day_warning_on(date(2026, 11, 3), TODAY);

    // Then
    assert!(warns);
}
