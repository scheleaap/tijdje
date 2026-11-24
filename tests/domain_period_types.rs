mod common;

use common::{day, part_of_day_period, period};
use tijdje::domain::{
    PartOfDay::FullDay,
    Period,
    PeriodType::{Break, Free, Holiday, Sick, Vacation, Work},
    TimeOfDay,
};

fn empty() -> Period {
    Period::default()
}

#[test]
fn toggling_a_type_checks_it_and_toggling_again_unchecks_it() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1230), Work),
    ]);
    d.toggle_period_type(1, Break);
    assert_eq!(d.periods[1].period_type, Break);
    d.toggle_period_type(1, Break);
    assert_eq!(d.periods[1].period_type, Work);
}

#[test]
fn types_are_mutually_exclusive() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1230), Break),
    ]);
    d.toggle_period_type(1, Sick);
    assert_eq!(d.periods[1].period_type, Sick);
}

#[test]
fn checking_break_does_not_fill_times() {
    let mut d = day(vec![empty()]);
    d.toggle_period_type(0, Break);
    assert_eq!((d.periods[0].from(), d.periods[0].to()), (None, None));
}

#[test]
fn no_fill_when_the_day_has_more_periods_or_times_are_set() {
    let mut d = day(vec![empty(), empty()]);
    d.toggle_period_type(0, Vacation);
    assert_eq!(d.periods[0].from(), None);

    let mut d = day(vec![period(800, None, Work)]);
    d.toggle_period_type(0, Vacation);
    assert_eq!(
        (d.periods[0].from(), d.periods[0].to()),
        (TimeOfDay::new(8, 0), None)
    );
}

#[test]
fn unchecking_keeps_times_that_were_changed() {
    let mut d = day(vec![period(900, Some(1300), Vacation)]);
    d.toggle_period_type(0, Vacation);
    assert_eq!(
        (d.periods[0].from(), d.periods[0].to()),
        (TimeOfDay::new(9, 0), TimeOfDay::new(13, 0))
    );
}

#[test]
fn switching_from_an_absence_to_break_keeps_the_times() {
    let mut d = day(vec![period(900, Some(1612), Holiday)]);
    d.toggle_period_type(0, Break);
    assert_eq!(d.periods[0], period(900, Some(1612), Break));
}

#[test]
fn switching_from_an_absence_to_break_keeps_the_full_day() {
    // Given
    let mut d = day(vec![empty()]);
    d.toggle_period_type(0, Holiday);

    // When
    d.toggle_period_type(0, Break);

    // Then
    assert_eq!(d.periods[0], part_of_day_period(FullDay, Break));
}

#[test]
fn checking_free_does_not_fill_times() {
    // Given
    let mut d = day(vec![empty()]);

    // When
    d.toggle_period_type(0, Free);

    // Then
    assert_eq!(d.periods[0].period_type, Free);
    assert_eq!((d.periods[0].from(), d.periods[0].to()), (None, None));
}

#[test]
fn free_is_mutually_exclusive_with_the_other_types() {
    // Given
    let mut d = day(vec![period(900, Some(1200), Break)]);

    // When
    d.toggle_period_type(0, Free);
    let after_checking_free = d.periods[0].period_type;
    d.toggle_period_type(0, Vacation);

    // Then
    assert_eq!(after_checking_free, Free);
    assert_eq!(d.periods[0].period_type, Vacation);
}
