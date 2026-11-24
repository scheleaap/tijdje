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
        ("vr  6 nov", "0h00m", "+6h09m"),   // full vacation day
        ("ma  9 nov", "+0h40m", "+6h49m"),  // 7h52m
        ("di 10 nov", "-3h22m", "+3h27m"),  // 3h50m, open period not counted
        ("wo 11 nov", "-7h12m", "-3h45m"),  // empty
        ("do 12 nov", "+0h24m", "-3h21m"),  // 4h00m worked, 3h36m sick
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
fn emoji_columns_line_up() {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let buffer = render(&mut app, 120, 40);
    let screen = rows(&buffer);
    let emoji_column = |day: &str, emoji: &str| {
        let y = row_index(&screen, day);
        (0..buffer.area.width)
            .find(|&x| buffer[(x, u16::try_from(y).unwrap())].symbol() == emoji)
            .unwrap()
    };
    assert_eq!(
        emoji_column("vr  6 nov", "🏖️"),
        emoji_column("do 12 nov", "🤒")
    );
    assert_eq!(
        emoji_column("di 10 nov", "⚠️"),
        emoji_column("wo 11 nov", "⚠️")
    );
}
