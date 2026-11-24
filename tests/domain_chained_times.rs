mod common;

use common::{day, period};
use tijdje::domain::{Period, PeriodType::Work, TimeOfDay};

fn time(hhmm: u16) -> Option<TimeOfDay> {
    TimeOfDay::new(hhmm / 100, hhmm % 100)
}

#[test]
fn changing_a_to_changes_the_next_from_that_was_equal_to_it() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1300), Work),
    ]);
    d.set_to(0, time(1215));
    assert_eq!(d.periods[0].to, time(1215));
    assert_eq!(d.periods[1].from, time(1215));
}

#[test]
fn changing_a_to_leaves_a_next_from_that_was_different() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1230, Some(1300), Work),
    ]);
    d.set_to(0, time(1215));
    assert_eq!(d.periods[1].from, time(1230));
}

#[test]
fn only_the_next_period_is_chained() {
    let mut d = day(vec![
        period(900, Some(1000), Work),
        period(1000, Some(1100), Work),
        period(1100, Some(1200), Work),
    ]);
    d.set_to(0, time(1030));
    assert_eq!(d.periods[1].to, time(1100));
    assert_eq!(d.periods[2].from, time(1100));
}

#[test]
fn changing_the_last_periods_to_changes_nothing_else() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1300), Work),
    ]);
    d.set_to(1, time(1330));
    assert_eq!(d.periods[0], period(900, Some(1200), Work));
    assert_eq!(d.periods[1].to, time(1330));
}

#[test]
fn filling_in_the_to_of_an_open_period_leaves_an_empty_next_from() {
    // The empty old To equals the empty next From, so an implementation that
    // compares them as optional values would wrongly fill in that From.
    let mut d = day(vec![period(900, None, Work), Period::default()]);
    d.set_to(0, time(1200));
    assert_eq!(d.periods[1].from, None);
}

#[test]
fn moving_a_to_past_the_next_periods_to_still_chains() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1300), Work),
    ]);
    d.set_to(0, time(1400));
    assert_eq!(d.periods[1].from, time(1400));
}

#[test]
fn clearing_a_to_leaves_the_next_from() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1300), Work),
    ]);
    d.set_to(0, None);
    assert_eq!(d.periods[0].to, None);
    assert_eq!(d.periods[1].from, time(1200));
}

#[test]
fn changing_a_from_changes_nothing_else() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1300), Work),
    ]);
    d.set_from(1, time(1215));
    assert_eq!(d.periods[0], period(900, Some(1200), Work));
    assert_eq!(d.periods[1].from, time(1215));
}

#[test]
fn setting_the_to_of_a_missing_period_does_nothing() {
    let mut d = day(vec![period(900, Some(1200), Work)]);
    d.set_to(1, time(1300));
    assert_eq!(d.periods, vec![period(900, Some(1200), Work)]);
}

#[test]
fn setting_the_from_of_a_missing_period_does_nothing() {
    let mut d = day(vec![period(900, Some(1200), Work)]);
    d.set_from(1, time(1300));
    assert_eq!(d.periods, vec![period(900, Some(1200), Work)]);
}
