mod common;

use common::{day, part_of_day_period, period};
use tijdje::domain::{
    PartOfDay::FirstHalf,
    Period,
    PeriodType::{Break, Free, Vacation, Work},
    TimeOfDay,
};

fn time(hhmm: u16) -> Option<TimeOfDay> {
    TimeOfDay::new(hhmm / 100, hhmm % 100)
}

fn work_until(periods: Vec<Period>) -> Option<TimeOfDay> {
    day(periods).work_until()
}

#[test]
fn work_until_adds_the_remaining_time_and_the_missing_break_time() {
    // Given: 5h30m worked with a 30m implied break.
    let periods = vec![
        period(900, Some(1200), Work),
        period(1230, Some(1500), Work),
    ];

    // When
    let until = work_until(periods);

    // Then: 1500 + 2h30m + 30m.
    assert_eq!(until, time(1800));
}

#[test]
fn a_whole_hour_of_break_leaves_no_missing_break_time() {
    // Given: 6h worked, a 30m explicit and a 30m implied break.
    let periods = vec![
        period(800, Some(1200), Work),
        period(1200, Some(1230), Break),
        period(1300, Some(1500), Work),
    ];

    // When
    let until = work_until(periods);

    // Then: 1500 + 2h.
    assert_eq!(until, time(1700));
}

#[test]
fn more_than_an_hour_of_break_doesnt_bring_work_until_forward() {
    // Given: 5h worked with a 2h implied break.
    let periods = vec![
        period(800, Some(1100), Work),
        period(1300, Some(1500), Work),
    ];

    // When
    let until = work_until(periods);

    // Then: 1500 + 3h.
    assert_eq!(until, time(1800));
}

#[test]
fn without_any_break_a_whole_hour_is_missing() {
    // Given: 4h worked without a break.
    let periods = vec![period(900, Some(1300), Work)];

    // When
    let until = work_until(periods);

    // Then: 1300 + 4h + 1h.
    assert_eq!(until, time(1800));
}

#[test]
fn free_periods_dont_count_as_break() {
    // Given: 4h worked, then 1h free directly after it.
    let periods = vec![
        period(800, Some(1200), Work),
        period(1200, Some(1300), Free),
    ];

    // When
    let until = work_until(periods);

    // Then: 1300 + 4h + 1h.
    assert_eq!(until, time(1800));
}

#[test]
fn an_open_last_period_is_the_base() {
    // Given: 3h worked, then an open period from 1230.
    let periods = vec![period(900, Some(1200), Work), period(1230, None, Work)];

    // When
    let until = work_until(periods);

    // Then: 1230 + 5h + 30m.
    assert_eq!(until, time(1800));
}

#[test]
fn periods_are_taken_in_time_order() {
    // Given: the example's periods in reverse order on screen.
    let periods = vec![
        period(1230, Some(1500), Work),
        period(900, Some(1200), Work),
    ];

    // When
    let until = work_until(periods);

    // Then: the same as in time order, with the 30m implied break.
    assert_eq!(until, time(1800));
}

#[test]
fn part_of_day_work_counts_as_worked_time() {
    // Given: 3h36m worked as a 1st half, and 2h from–to.
    let periods = vec![
        part_of_day_period(FirstHalf, Work),
        period(1300, Some(1500), Work),
    ];

    // When
    let until = work_until(periods);

    // Then: 1500 + 2h24m + 1h.
    assert_eq!(until, time(1824));
}

#[test]
fn absences_dont_count_as_worked_time() {
    // Given: 3h worked, a 1h implied break and 2h vacation.
    let periods = vec![
        period(900, Some(1200), Work),
        period(1300, Some(1500), Vacation),
    ];

    // When
    let until = work_until(periods);

    // Then: 1500 + 5h.
    assert_eq!(until, time(2000));
}

#[test]
fn there_is_no_work_until_once_8h_is_worked() {
    // Given: 8h worked.
    let periods = vec![
        period(800, Some(1200), Work),
        period(1230, Some(1630), Work),
    ];

    // When
    let until = work_until(periods);

    // Then
    assert_eq!(until, None);
}

#[test]
fn there_is_no_work_until_without_from_to_periods() {
    assert_eq!(work_until(vec![part_of_day_period(FirstHalf, Work)]), None);
    assert_eq!(work_until(vec![]), None);
    assert_eq!(work_until(vec![Period::default()]), None);
}

#[test]
fn work_until_can_be_late_in_the_evening() {
    // Given: 1h worked, ending at 1500.
    let periods = vec![period(1400, Some(1500), Work)];

    // When
    let until = work_until(periods);

    // Then: 1500 + 7h + 1h.
    assert_eq!(until, time(2300));
}

#[test]
fn there_is_no_work_until_at_midnight_or_later() {
    // Given: 1h worked, ending at 1600: 1600 + 7h + 1h is 24:00.
    let periods = vec![period(1500, Some(1600), Work)];

    // When
    let until = work_until(periods);

    // Then
    assert_eq!(until, None);
}
