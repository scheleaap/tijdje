mod common;

use common::{day, period};
use tijdje::domain::{
    Period,
    PeriodType::{self, Break, Holiday, Sick, Vacation, Work},
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
fn checking_an_absence_on_a_lone_empty_period_fills_a_full_day() {
    for absence in [Vacation, Sick, Holiday] {
        let mut d = day(vec![empty()]);
        d.toggle_period_type(0, absence);
        assert_eq!(d.periods[0].from, TimeOfDay::new(9, 0), "{absence:?}");
        assert_eq!(d.periods[0].to, TimeOfDay::new(16, 12), "{absence:?}");
    }
}

#[test]
fn the_full_day_follows_the_regular_expected_working_time() {
    let mut d = day(vec![empty()]);
    d.regular_expected_working_time = Some(tijdje::domain::Minutes(240));
    d.toggle_period_type(0, Vacation);
    assert_eq!(d.periods[0].to, TimeOfDay::new(13, 0));
}

#[test]
fn checking_break_does_not_fill_times() {
    let mut d = day(vec![empty()]);
    d.toggle_period_type(0, Break);
    assert_eq!((d.periods[0].from, d.periods[0].to), (None, None));
}

#[test]
fn no_fill_when_the_day_has_more_periods_or_times_are_set() {
    let mut d = day(vec![empty(), empty()]);
    d.toggle_period_type(0, Vacation);
    assert_eq!(d.periods[0].from, None);

    let mut d = day(vec![period(800, None, Work)]);
    d.toggle_period_type(0, Vacation);
    assert_eq!(
        (d.periods[0].from, d.periods[0].to),
        (TimeOfDay::new(8, 0), None)
    );
}

#[test]
fn unchecking_an_auto_filled_absence_empties_the_times() {
    let mut d = day(vec![empty()]);
    d.toggle_period_type(0, Sick);
    d.toggle_period_type(0, Sick);
    assert_eq!(d.periods[0].period_type, Work);
    assert_eq!((d.periods[0].from, d.periods[0].to), (None, None));
}

#[test]
fn unchecking_keeps_times_that_were_changed() {
    let mut d = day(vec![period(900, Some(1300), Vacation)]);
    d.toggle_period_type(0, Vacation);
    assert_eq!(
        (d.periods[0].from, d.periods[0].to),
        (TimeOfDay::new(9, 0), TimeOfDay::new(13, 0))
    );
}

#[test]
fn switching_between_absences_keeps_the_times() {
    let mut d = day(vec![empty()]);
    d.toggle_period_type(0, Vacation);
    d.toggle_period_type(0, Sick);
    assert_eq!(d.periods[0].period_type, Sick);
    assert_eq!(
        (d.periods[0].from, d.periods[0].to),
        (TimeOfDay::new(9, 0), TimeOfDay::new(16, 12))
    );
}

#[test]
fn switching_from_an_absence_to_break_keeps_the_times() {
    let mut d = day(vec![empty()]);
    d.toggle_period_type(0, Holiday);
    d.toggle_period_type(0, Break);
    assert_eq!(d.periods[0].period_type, PeriodType::Break);
    assert_eq!(d.periods[0].from, TimeOfDay::new(9, 0));
}
