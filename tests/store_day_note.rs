mod common;

use std::{collections::BTreeMap, path::Path};

use common::{InMemoryFileSystem, period};
use jiff::civil::date;
use tijdje::{
    domain::{Day, Minutes, PeriodType, Timesheet},
    store::{self, Backup, Overwrite},
};

const PATH: &str = "t.yaml";

fn load(content: &str) -> Timesheet {
    store::load(&InMemoryFileSystem::with(PATH, content), Path::new(PATH))
        .expect("valid timesheet")
        .0
}

/// Saves `timesheet` to an empty in-memory file and returns the YAML.
fn save(timesheet: &Timesheet) -> String {
    let fs = InMemoryFileSystem::default();
    let (_, fingerprint) = store::load(&fs, Path::new(PATH)).unwrap();
    store::save(
        &fs,
        Path::new(PATH),
        timesheet,
        &fingerprint,
        Overwrite::IfUnchanged,
        Backup::Skip,
    )
    .unwrap();
    fs.get(PATH).unwrap()
}

fn timesheet_with(day: Day) -> Timesheet {
    Timesheet {
        days: BTreeMap::from([(date(2026, 11, 12), day)]),
    }
}

#[test]
fn reads_the_day_note() {
    // Given
    let yaml = "days:\n  \"2026-11-12\":\n    regular_expected_working_time_in_minutes: 432\n    notes: \"Team day\"\n    periods: []\n";

    // When
    let timesheet = load(yaml);

    // Then
    assert_eq!(
        timesheet.days[&date(2026, 11, 12)].day_note.as_deref(),
        Some("Team day")
    );
}

#[test]
fn reads_an_empty_or_missing_day_note_as_none() {
    // Given
    let yaml = "days:\n  \"2026-11-12\":\n    regular_expected_working_time_in_minutes: 432\n    notes: \"\"\n  \"2026-11-13\":\n    regular_expected_working_time_in_minutes: 432\n";

    // When
    let timesheet = load(yaml);

    // Then
    assert_eq!(timesheet.days[&date(2026, 11, 12)].day_note, None);
    assert_eq!(timesheet.days[&date(2026, 11, 13)].day_note, None);
}

#[test]
fn writes_the_day_note_after_the_regular_expected_working_time() {
    // Given
    let timesheet = timesheet_with(Day {
        regular_expected_working_time: Some(Minutes(432)),
        day_note: Some("Team \"day\"".to_owned()),
        periods: vec![period(900, Some(1200), PeriodType::Work)],
    });

    // When
    let yaml = save(&timesheet);

    // Then
    assert!(
        yaml.contains("    regular_expected_working_time_in_minutes: 432\n    notes: \"Team \\\"day\\\"\"\n    periods:\n      - from: \"09:00\"\n"),
        "{yaml}"
    );
    assert_eq!(load(&yaml), timesheet);
}

#[test]
fn writes_an_empty_day_note_as_null() {
    // Given
    let timesheet = timesheet_with(Day {
        periods: vec![period(900, Some(1200), PeriodType::Work)],
        ..Day::default()
    });

    // When
    let yaml = save(&timesheet);

    // Then
    assert!(yaml.contains("    notes: null\n    periods:\n"), "{yaml}");
}

#[test]
fn writes_a_day_with_only_a_day_note() {
    // Given
    let timesheet = timesheet_with(Day {
        regular_expected_working_time: Some(Minutes(432)),
        day_note: Some("Team day".to_owned()),
        periods: vec![],
    });

    // When
    let yaml = save(&timesheet);

    // Then
    assert!(
        yaml.contains("  \"2026-11-12\":\n    regular_expected_working_time_in_minutes: 432\n    notes: \"Team day\"\n    periods: []\n"),
        "{yaml}"
    );
    assert_eq!(load(&yaml), timesheet);
}
