mod common;

use common::period;
use tijdje::domain::{Absence, Day, Minutes, Period, PeriodType};

fn day(periods: Vec<Period>) -> Day {
    Day {
        regular_expected_working_time: Some(Minutes(432)),
        periods,
    }
}

#[test]
fn actual_expected_equals_regular_without_absences() {
    let d = day(vec![period(900, Some(1200), PeriodType::Work)]);
    assert_eq!(d.actual_expected_working_time(), Minutes(432));
}

#[test]
fn vacation_sick_and_holiday_reduce_actual_expected() {
    let d = day(vec![
        period(800, Some(900), PeriodType::Vacation),
        period(900, Some(1000), PeriodType::Sick),
        period(1000, Some(1030), PeriodType::Holiday),
        period(1030, Some(1100), PeriodType::Break),
    ]);
    assert_eq!(d.actual_expected_working_time(), Minutes(432 - 150));
}

#[test]
fn regular_expected_defaults_to_7h12m() {
    let d = Day {
        regular_expected_working_time: None,
        periods: vec![],
    };
    assert_eq!(d.actual_expected_working_time(), Minutes(432));
}

#[test]
fn day_balance_is_worked_minus_actual_expected() {
    // 4h00m worked, 3h36m sick: actual expected 3h36m, balance +0h24m.
    let d = day(vec![
        period(800, Some(1200), PeriodType::Work),
        period(1200, Some(1536), PeriodType::Sick),
    ]);
    assert_eq!(d.day_balance(), Minutes(24));
}

#[test]
fn full_vacation_day_has_zero_balance() {
    let d = day(vec![period(900, Some(1612), PeriodType::Vacation)]);
    assert_eq!(d.day_balance(), Minutes(0));
}

#[test]
fn absence_is_none_for_work_and_breaks() {
    let d = day(vec![
        period(900, Some(1200), PeriodType::Work),
        period(1200, Some(1230), PeriodType::Break),
    ]);
    assert_eq!(d.absence(), None);
}

#[test]
fn sick_wins_over_vacation_and_holiday() {
    let d = day(vec![
        period(800, Some(1000), PeriodType::Holiday),
        period(1000, Some(1200), PeriodType::Vacation),
        period(1200, Some(1400), PeriodType::Sick),
    ]);
    assert_eq!(d.absence(), Some(Absence::Sick));
}

#[test]
fn vacation_wins_over_holiday() {
    let d = day(vec![
        period(800, Some(1000), PeriodType::Holiday),
        period(1000, Some(1200), PeriodType::Vacation),
    ]);
    assert_eq!(d.absence(), Some(Absence::Vacation));
}

#[test]
fn holiday_alone_is_shown() {
    let d = day(vec![period(900, Some(1612), PeriodType::Holiday)]);
    assert_eq!(d.absence(), Some(Absence::Holiday));
}
