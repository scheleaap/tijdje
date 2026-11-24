mod common;

use std::collections::BTreeMap;

use common::{day, period};
use jiff::civil::{Date, date};
use tijdje::domain::{
    Absence, DayIndicator, Period,
    PeriodType::{Holiday, Sick, Vacation, Work},
    Timesheet,
};

const MONDAY: Date = date(2026, 11, 2);
const TUESDAY: Date = date(2026, 11, 3);
const TODAY: Date = date(2026, 11, 5);

/// A timesheet with only `periods` on Monday 2 Nov.
fn monday_with(periods: Vec<Period>) -> Timesheet {
    Timesheet {
        days: BTreeMap::from([(MONDAY, day(periods))]),
    }
}

#[test]
fn a_consistent_working_day_has_no_day_indicator() {
    // Given
    let t = monday_with(vec![period(900, Some(1612), Work)]);

    // When
    let indicator = t.day_indicator_on(MONDAY, TODAY);

    // Then
    assert_eq!(indicator, None);
}

#[test]
fn a_day_with_an_absence_shows_the_absence_indicator() {
    // Given
    let t = monday_with(vec![period(900, Some(1612), Vacation)]);

    // When
    let indicator = t.day_indicator_on(MONDAY, TODAY);

    // Then
    assert_eq!(indicator, Some(DayIndicator::Absence(Absence::Vacation)));
}

#[test]
fn the_absence_indicator_prefers_sick_over_vacation_over_holiday() {
    // Given
    let t = monday_with(vec![
        period(900, Some(1000), Holiday),
        period(1000, Some(1100), Vacation),
        period(1100, Some(1200), Sick),
    ]);

    // When
    let indicator = t.day_indicator_on(MONDAY, TODAY);

    // Then
    assert_eq!(indicator, Some(DayIndicator::Absence(Absence::Sick)));
}

#[test]
fn an_empty_past_day_shows_the_day_warning() {
    // Given Tuesday is empty, after the first recorded day and before today
    let t = monday_with(vec![period(900, Some(1612), Work)]);

    // When
    let indicator = t.day_indicator_on(TUESDAY, TODAY);

    // Then
    assert_eq!(indicator, Some(DayIndicator::DayWarning));
}

#[test]
fn an_inconsistent_day_with_an_absence_shows_the_day_warning() {
    // Given a sick period overlapping a work period
    let t = monday_with(vec![
        period(900, Some(1200), Work),
        period(1100, Some(1612), Sick),
    ]);

    // When
    let indicator = t.day_indicator_on(MONDAY, TODAY);

    // Then
    assert_eq!(indicator, Some(DayIndicator::DayWarning));
}
