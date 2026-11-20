mod common;

use std::collections::BTreeMap;

use common::period;
use jiff::civil::{Date, date};
use tijdje::domain::{Day, Minutes, Period, PeriodType, Timesheet};

/// Mon 2 Nov: +1h00m (8h12m worked); Tue 3 Nov: empty; Wed 4 Nov: -0h12m (7h00m worked).
fn timesheet() -> Timesheet {
    let day = |p: Period| Day {
        regular_expected_working_time: Some(Minutes(432)),
        periods: vec![p],
    };
    Timesheet {
        days: BTreeMap::from([
            (
                date(2026, 11, 2),
                day(period(800, Some(1612), PeriodType::Work)),
            ),
            (
                date(2026, 11, 4),
                day(period(900, Some(1600), PeriodType::Work)),
            ),
        ]),
    }
}

const TODAY: Date = date(2026, 11, 5);

#[test]
fn recorded_day_has_its_own_day_balance() {
    assert_eq!(
        timesheet().day_balance_on(date(2026, 11, 2), TODAY),
        Some(Minutes(60))
    );
}

#[test]
fn empty_day_after_the_first_recorded_day_counts_full_regular_expected() {
    assert_eq!(
        timesheet().day_balance_on(date(2026, 11, 3), TODAY),
        Some(Minutes(-432))
    );
}

#[test]
fn no_balances_before_the_first_recorded_day() {
    let t = timesheet();
    assert_eq!(t.day_balance_on(date(2026, 10, 30), TODAY), None);
    assert_eq!(t.total_balance_on(date(2026, 10, 30), TODAY), None);
}

#[test]
fn no_balances_for_future_days() {
    let t = timesheet();
    assert_eq!(t.day_balance_on(date(2026, 11, 6), TODAY), None);
    assert_eq!(t.total_balance_on(date(2026, 11, 6), TODAY), None);
}

#[test]
fn today_has_balances() {
    // Thu 5 Nov is empty: -7h12m.
    assert_eq!(
        timesheet().day_balance_on(TODAY, TODAY),
        Some(Minutes(-432))
    );
}

#[test]
fn total_balance_adds_up_from_the_first_recorded_day() {
    let t = timesheet();
    assert_eq!(
        t.total_balance_on(date(2026, 11, 2), TODAY),
        Some(Minutes(60))
    );
    assert_eq!(
        t.total_balance_on(date(2026, 11, 3), TODAY),
        Some(Minutes(60 - 432))
    );
    assert_eq!(
        t.total_balance_on(date(2026, 11, 4), TODAY),
        Some(Minutes(60 - 432 - 12))
    );
    assert_eq!(
        t.total_balance_on(TODAY, TODAY),
        Some(Minutes(60 - 432 - 12 - 432))
    );
}

#[test]
fn total_balance_skips_weekends() {
    let mut t = timesheet();
    t.days.clear();
    t.days.insert(
        date(2026, 11, 6),
        Day {
            regular_expected_working_time: Some(Minutes(432)),
            periods: vec![period(900, Some(1612), PeriodType::Work)],
        },
    );
    // Fri 6 Nov: 0; Mon 9 Nov: empty, -7h12m. The weekend doesn't count.
    assert_eq!(
        t.total_balance_on(date(2026, 11, 9), date(2026, 11, 9)),
        Some(Minutes(-432))
    );
}

#[test]
fn empty_timesheet_has_no_balances() {
    assert_eq!(Timesheet::default().total_balance_on(TODAY, TODAY), None);
}

fn with_open_period(mut t: Timesheet, on: Date) -> Timesheet {
    // After the day's last period, so it doesn't overlap.
    t.days
        .get_mut(&on)
        .unwrap()
        .periods
        .push(period(1700, None, PeriodType::Work));
    t
}

#[test]
fn empty_past_day_after_the_first_recorded_day_is_warned() {
    assert!(timesheet().has_day_warning_on(date(2026, 11, 3), TODAY));
}

#[test]
fn recorded_past_day_is_not_warned() {
    assert!(!timesheet().has_day_warning_on(date(2026, 11, 4), TODAY));
}

#[test]
fn past_day_with_an_open_period_is_warned() {
    let t = with_open_period(timesheet(), date(2026, 11, 2));
    assert!(t.has_day_warning_on(date(2026, 11, 2), TODAY));
}

#[test]
fn today_is_never_warned_for_being_empty_or_open() {
    let t = with_open_period(timesheet(), date(2026, 11, 4));
    assert!(!timesheet().has_day_warning_on(TODAY, TODAY));
    assert!(!t.has_day_warning_on(date(2026, 11, 4), date(2026, 11, 4)));
}

#[test]
fn days_before_the_first_recorded_day_and_future_days_are_not_warned() {
    let t = timesheet();
    assert!(!t.has_day_warning_on(date(2026, 10, 30), TODAY));
    assert!(!t.has_day_warning_on(date(2026, 11, 6), TODAY));
}

fn with_periods(mut t: Timesheet, on: Date, periods: Vec<Period>) -> Timesheet {
    t.day_mut(on).periods.extend(periods);
    t
}

#[test]
fn a_day_failing_the_consistency_check_is_warned_wherever_it_is() {
    let overlap = || {
        vec![
            period(900, Some(1200), PeriodType::Work),
            period(1100, Some(1300), PeriodType::Work),
        ]
    };
    for on in [
        date(2026, 10, 30),
        date(2026, 11, 3),
        TODAY,
        date(2026, 11, 6),
    ] {
        let t = with_periods(timesheet(), on, overlap());
        assert!(t.has_day_warning_on(on, TODAY), "{on}");
    }
}

#[test]
fn a_past_day_with_only_empty_periods_counts_as_empty() {
    let t = with_periods(timesheet(), date(2026, 11, 3), vec![Period::default()]);
    assert!(t.has_day_warning_on(date(2026, 11, 3), TODAY));
}

#[test]
fn a_day_with_only_empty_periods_is_not_the_first_recorded_day() {
    let t = with_periods(timesheet(), date(2026, 10, 30), vec![Period::default()]);
    assert_eq!(t.first_recorded_day(), Some(date(2026, 11, 2)));
    assert!(!t.has_day_warning_on(date(2026, 10, 30), TODAY));
}

#[test]
fn an_empty_period_next_to_others_does_not_warn_the_day() {
    let t = with_periods(timesheet(), date(2026, 11, 4), vec![Period::default()]);
    assert!(!t.has_day_warning_on(date(2026, 11, 4), TODAY));
}

#[test]
fn a_period_warning_does_not_warn_the_day() {
    let t = with_periods(
        timesheet(),
        date(2026, 11, 3),
        vec![
            period(500, Some(1200), PeriodType::Work),
            period(1200, Some(1300), PeriodType::Work),
        ],
    );
    assert!(!t.has_day_warning_on(date(2026, 11, 3), TODAY));
}
