mod common;

use common::{day, period};
use tijdje::domain::{Absence, Day, Minutes, PeriodType};

/// 3h00m work and 2h00m free.
fn day_with_a_free_period() -> Day {
    day(vec![
        period(900, Some(1200), PeriodType::Work),
        period(1300, Some(1500), PeriodType::Free),
    ])
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
fn free_periods_do_not_reduce_actual_expected() {
    // Given
    let d = day_with_a_free_period();

    // When
    let actual = d.actual_expected_working_time();

    // Then
    assert_eq!(actual, Minutes(432));
}

#[test]
fn regular_expected_defaults_to_7h12m() {
    let d = Day {
        regular_expected_working_time: None,
        periods: vec![],
        ..Day::default()
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
fn a_free_period_lowers_the_day_balance() {
    // Given: 3h00m worked, 7h12m expected.
    let d = day_with_a_free_period();

    // When
    let balance = d.day_balance();

    // Then
    assert_eq!(balance, Minutes(180 - 432));
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
fn absence_is_none_for_free_periods() {
    // Given
    let d = day_with_a_free_period();

    // When
    let absence = d.absence();

    // Then
    assert_eq!(absence, None);
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

#[test]
fn absence_time_sums_vacation_sick_and_holiday() {
    let d = day(vec![
        period(800, Some(1000), PeriodType::Work),
        period(1000, Some(1030), PeriodType::Break),
        period(1030, Some(1230), PeriodType::Sick),
        period(1230, Some(1330), PeriodType::Vacation),
    ]);
    assert_eq!(d.absence_time(), Minutes(180));
}

#[test]
fn absence_time_excludes_free_periods() {
    // Given
    let d = day_with_a_free_period();

    // When
    let absence_time = d.absence_time();

    // Then
    assert_eq!(absence_time, Minutes(0));
}
