mod common;

use common::{day, period};
use tijdje::domain::PeriodType::Work;

#[test]
fn gap_before_the_next_period_is_an_implied_break() {
    let d = day(vec![
        period(900, Some(1200), Work),
        period(1230, Some(1700), Work),
    ]);
    assert!(d.has_implied_break_after(0));
}

#[test]
fn consecutive_periods_have_no_implied_break() {
    let d = day(vec![
        period(900, Some(1200), Work),
        period(1200, Some(1700), Work),
    ]);
    assert!(!d.has_implied_break_after(0));
}

#[test]
fn last_period_has_no_implied_break() {
    let d = day(vec![
        period(900, Some(1200), Work),
        period(1230, Some(1700), Work),
    ]);
    assert!(!d.has_implied_break_after(1));
}

#[test]
fn open_period_has_no_implied_break() {
    let d = day(vec![
        period(900, None, Work),
        period(1230, Some(1700), Work),
    ]);
    assert!(!d.has_implied_break_after(0));
}

#[test]
fn start_before_six_or_end_after_eight_is_unusual() {
    assert!(period(559, Some(1200), Work).has_unusual_time());
    assert!(period(1200, Some(2001), Work).has_unusual_time());
    assert!(!period(600, Some(2000), Work).has_unusual_time());
    assert!(!period(900, None, Work).has_unusual_time());
}

#[test]
fn new_period_starts_where_the_last_one_ended() {
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1230, Some(1500), Work),
    ]);
    let index = d.add_period();
    assert_eq!(index, 2);
    assert_eq!(d.periods[2], period(1500, None, Work));
}

#[test]
fn new_period_after_an_open_period_has_an_empty_from() {
    let mut d = day(vec![period(900, None, Work)]);
    d.add_period();
    assert_eq!(d.periods[1].from, None);
}

#[test]
fn first_period_of_an_empty_day_has_an_empty_from() {
    let mut d = day(vec![]);
    assert_eq!(d.add_period(), 0);
    assert_eq!(d.periods[0].from, None);
    assert_eq!(d.periods[0].to, None);
}

#[test]
fn a_period_without_any_values_is_empty() {
    use tijdje::domain::{Period, PeriodType};
    assert!(Period::default().is_empty());
    assert!(!period(900, None, Work).is_empty());
    let with_notes = Period {
        notes: Some("x".into()),
        ..Period::default()
    };
    assert!(!with_notes.is_empty());
    let a_break = Period {
        period_type: PeriodType::Break,
        ..Period::default()
    };
    assert!(!a_break.is_empty());
}
