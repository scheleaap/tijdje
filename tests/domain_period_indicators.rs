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
