mod common;

use common::*;
use jiff::civil::date;

/// Worked out by hand from the sample file, with today = Fri 13 Nov 2026 and a
/// regular expected working time of 7h12m (432 minutes) per day.
#[test]
fn left_panel_shows_day_and_total_balances() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 40));
    for (day, day_balance, total_balance) in [
        ("ma  2 nov", "+1h30m", "+1h30m"),  // 8h42m worked
        ("di  3 nov", "+1h26m", "+2h56m"),  // 8h38m
        ("wo  4 nov", "+1h32m", "+4h28m"),  // 8h44m
        ("do  5 nov", "+1h41m", "+6h09m"),  // 8h53m
        ("vr  6 nov", "0m", "+6h09m"),      // full vacation day
        ("ma  9 nov", "+40m", "+6h49m"),    // 7h52m
        ("di 10 nov", "-3h22m", "+3h27m"),  // 3h50m, open period not counted
        ("wo 11 nov", "-7h12m", "-3h45m"),  // empty
        ("do 12 nov", "+24m", "-3h21m"),    // 4h00m worked, 3h36m sick
        ("vr 13 nov", "-7h12m", "-10h33m"), // today, empty
    ] {
        let row = row_containing(&screen, day);
        let columns: Vec<&str> = row.split_whitespace().collect();
        assert!(
            columns.contains(&day_balance) && columns.contains(&total_balance),
            "{day} should show {day_balance} / {total_balance}: {row:?}"
        );
    }
}

#[test]
fn future_days_and_days_before_the_first_recorded_day_show_no_balances() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 40));
    for day in ["vr 30 okt", "ma 16 nov", "di 17 nov"] {
        let row = row_containing(&screen, day);
        assert!(!row.contains('h'), "{day} should show no figures: {row:?}");
    }
}

#[test]
fn absence_indicators_show_per_day() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 40));
    assert!(row_containing(&screen, "vr  6 nov").contains("🏖️"));
    assert!(row_containing(&screen, "do 12 nov").contains("🤒"));
    assert!(!row_containing(&screen, "ma  9 nov").contains("🏖️"));

    let mut app = app_with(SAMPLE, date(2026, 12, 31));
    let screen = rows(&render(&mut app, 120, 40));
    assert!(row_containing(&screen, "vr  1 jan").contains("🎄"));
}

#[test]
fn warnings_show_for_past_empty_days_and_open_periods_only() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let screen = rows(&render(&mut app, 120, 40));
    assert!(
        row_containing(&screen, "di 10 nov").contains("⚠️"),
        "open period"
    );
    assert!(
        row_containing(&screen, "wo 11 nov").contains("⚠️"),
        "empty day"
    );
    for day in ["vr 30 okt", "ma  9 nov", "vr 13 nov", "ma 16 nov"] {
        assert!(
            !row_containing(&screen, day).contains("⚠️"),
            "{day} should have no warning"
        );
    }
}

#[test]
fn absence_indicators_and_day_warnings_share_one_column_after_the_total_balance() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 13));

    // When
    let buffer = render(&mut app, 120, 40);

    // Then
    let screen = rows(&buffer);
    let y = u16::try_from(row_index(&screen, "vr  6 nov")).unwrap();
    let total_balance_end = column_of(&buffer, y, "+6h09m").unwrap() + 6;
    let indicator_column = total_balance_end + 2;
    for (day, emoji) in [
        ("vr  6 nov", "🏖️"),
        ("do 12 nov", "🤒"),
        ("di 10 nov", "⚠️"),
        ("wo 11 nov", "⚠️"),
    ] {
        assert_eq!(
            symbol_column(&buffer, day, emoji),
            indicator_column,
            "{day}"
        );
    }
}

#[test]
fn an_inconsistent_day_with_an_absence_shows_only_the_day_warning() {
    // Given a sick period overlapping a work period on a past day
    let yaml = r#"
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: "12:00", project: acme, notes: null, type: work }
      - { from: "11:00", to: "16:12", project: null, notes: null, type: sick }
"#;
    let mut app = app_with(yaml, date(2026, 11, 3));

    // When
    let screen = rows(&render(&mut app, 120, 24));

    // Then
    let monday = left(row_containing(&screen, "ma  2 nov"));
    assert!(
        monday.contains("⚠️") && !monday.contains("🤒"),
        "{monday:?}"
    );
}

#[test]
fn a_day_with_a_free_period_shows_the_free_indicator() {
    // Given a past day with a work period and a free period
    let yaml = r#"
days:
  "2026-11-02":
    regular_expected_working_time_in_minutes: 432
    periods:
      - { from: "09:00", to: "13:00", project: acme, notes: null, type: work }
      - { from: "13:00", to: "16:12", project: null, notes: null, type: free }
"#;
    let mut app = app_with(yaml, date(2026, 11, 3));

    // When
    let screen = rows(&render(&mut app, 120, 24));

    // Then
    let monday = left(row_containing(&screen, "ma  2 nov"));
    assert!(monday.contains("🆓"), "{monday:?}");
}
