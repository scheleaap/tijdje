mod common;

use std::collections::BTreeMap;

use common::{day, load_timesheet, part_of_day_period, period, save_timesheet};
use jiff::civil::date;
use tijdje::{
    domain::{
        PartOfDay::{FirstHalf, FullDay, SecondHalf},
        PeriodType::{Vacation, Work},
        Timesheet,
    },
    store::LoadError,
};

/// A day in the file with one period whose `from`, `to` and `part_of_day`
/// keys have these values.
fn yaml_with(from: &str, to: &str, part_of_day: &str) -> String {
    format!(
        "days:\n  \"2026-11-12\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n      - from: {from}\n        to: {to}\n        part_of_day: {part_of_day}\n        project: null\n        notes: null\n        type: vacation\n"
    )
}

#[test]
fn reads_each_part_of_day() {
    for (name, part) in [
        ("full_day", FullDay),
        ("first_half", FirstHalf),
        ("second_half", SecondHalf),
    ] {
        // Given
        let yaml = yaml_with("null", "null", name);

        // When
        let timesheet = load_timesheet(&yaml).unwrap();

        // Then
        assert_eq!(
            timesheet.days[&date(2026, 11, 12)].periods,
            vec![part_of_day_period(part, Vacation)],
            "{name}"
        );
    }
}

#[test]
fn reads_a_missing_or_null_part_of_day_as_none() {
    // Given
    let missing = "days:\n  \"2026-11-12\":\n    regular_expected_working_time_in_minutes: 432\n    periods:\n      - { from: \"09:00\", to: \"12:00\", project: null, notes: null, type: work }\n";
    let null = yaml_with("\"09:00\"", "\"12:00\"", "null");

    // When
    let timesheets = [load_timesheet(missing), load_timesheet(&null)];

    // Then
    for timesheet in timesheets {
        assert_eq!(
            timesheet.unwrap().days[&date(2026, 11, 12)].periods[0].part_of_day(),
            None
        );
    }
}

#[test]
fn a_part_of_day_with_a_from_or_to_is_a_parse_error() {
    for (from, to) in [("\"09:00\"", "null"), ("null", "\"12:00\"")] {
        // Given
        let yaml = yaml_with(from, to, "full_day");

        // When
        let result = load_timesheet(&yaml);

        // Then
        assert!(
            matches!(result, Err(LoadError::Parse(_))),
            "{from} {to}: {result:?}"
        );
    }
}

#[test]
fn an_unknown_part_of_day_is_a_parse_error() {
    // Given
    let yaml = yaml_with("null", "null", "morning");

    // When
    let result = load_timesheet(&yaml);

    // Then
    assert!(matches!(result, Err(LoadError::Parse(_))), "{result:?}");
}

#[test]
fn writes_the_part_of_day_with_empty_times() {
    // Given
    let timesheet = Timesheet {
        days: BTreeMap::from([(
            date(2026, 11, 12),
            day(vec![
                part_of_day_period(FirstHalf, Vacation),
                period(1300, Some(1700), Work),
            ]),
        )]),
    };

    // When
    let yaml = save_timesheet(&timesheet);

    // Then
    assert!(
        yaml.contains(
            "      - from: null\n        to: null\n        part_of_day: first_half\n        project: null\n        notes: null\n        type: vacation\n"
        ),
        "{yaml}"
    );
    assert!(
        yaml.contains(
            "      - from: \"13:00\"\n        to: \"17:00\"\n        part_of_day: null\n"
        ),
        "{yaml}"
    );
}

#[test]
fn a_saved_part_of_day_loads_again() {
    // Given
    let timesheet = Timesheet {
        days: BTreeMap::from([(
            date(2026, 11, 12),
            day(vec![part_of_day_period(SecondHalf, Work)]),
        )]),
    };

    // When
    let loaded = load_timesheet(&save_timesheet(&timesheet)).unwrap();

    // Then
    assert_eq!(loaded, timesheet);
}
