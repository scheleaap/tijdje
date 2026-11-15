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
    let mut open = period(1300, Some(1400), PeriodType::Work);
    open.to = None;
    t.days.get_mut(&on).unwrap().periods.push(open);
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
