mod common;

use common::{day, period};
use jiff::civil::{Date, date};
use tijdje::domain::{
    Period, PeriodIndicator,
    PeriodIndicator::{ConsistencyFailure, NonWorking, PeriodWarning},
    PeriodType::{Break, Free, Vacation, Work},
};

const TODAY: Date = date(2026, 11, 13);
const PAST: Date = date(2026, 11, 12);

fn indicators(periods: Vec<Period>) -> Vec<Option<PeriodIndicator>> {
    day(periods).period_indicators(PAST, TODAY)
}

fn with_project_and_notes(mut p: Period, project: Option<&str>, notes: Option<&str>) -> Period {
    p.project = project.map(Into::into);
    p.notes = notes.map(Into::into);
    p
}

#[test]
fn an_ordinary_work_period_has_no_indicator() {
    assert_eq!(indicators(vec![period(900, Some(1200), Work)]), vec![None]);
}

#[test]
fn an_unusual_time_gets_a_period_warning() {
    assert_eq!(
        indicators(vec![
            period(559, Some(700), Work),
            period(1900, Some(2001), Work)
        ]),
        vec![Some(PeriodWarning), Some(PeriodWarning)]
    );
    assert_eq!(indicators(vec![period(600, Some(2000), Work)]), vec![None]);
}

#[test]
fn a_free_period_at_an_unusual_time_gets_a_period_warning() {
    // Given
    let periods = vec![period(1900, Some(2030), Free)];

    // When
    let found = indicators(periods);

    // Then
    assert_eq!(found, vec![Some(PeriodWarning)]);
}

#[test]
fn a_needless_split_warns_on_the_second_period() {
    assert_eq!(
        indicators(vec![
            with_project_and_notes(period(900, Some(1200), Work), Some("acme"), Some("x")),
            with_project_and_notes(period(1200, Some(1300), Work), Some("acme"), Some("x")),
        ]),
        vec![None, Some(PeriodWarning)]
    );
}

#[test]
fn a_needless_split_without_project_and_notes_is_warned_too() {
    assert_eq!(
        indicators(vec![
            period(900, Some(1200), Work),
            period(1200, Some(1300), Work),
        ]),
        vec![None, Some(PeriodWarning)]
    );
}

#[test]
fn adjacent_free_periods_with_the_same_notes_are_a_needless_split() {
    // Given
    let periods = vec![
        with_project_and_notes(period(1300, Some(1400), Free), None, Some("x")),
        with_project_and_notes(period(1400, Some(1500), Free), None, Some("x")),
    ];

    // When
    let found = indicators(periods);

    // Then
    assert_eq!(found, vec![Some(NonWorking(Free)), Some(PeriodWarning)]);
}

#[test]
fn consecutive_periods_that_differ_are_not_warned() {
    let first = || with_project_and_notes(period(900, Some(1200), Work), Some("acme"), Some("x"));
    for second in [
        with_project_and_notes(period(1200, Some(1300), Work), Some("other"), Some("x")),
        with_project_and_notes(period(1200, Some(1300), Work), Some("acme"), Some("y")),
        with_project_and_notes(period(1200, Some(1300), Work), Some("acme"), None),
        with_project_and_notes(period(1200, Some(1300), Work), None, Some("x")),
    ] {
        assert_eq!(
            indicators(vec![first(), second.clone()]),
            vec![None, None],
            "{second:?}"
        );
    }
    assert_eq!(
        indicators(vec![
            period(900, Some(1200), Work),
            period(1200, Some(1300), Break)
        ]),
        vec![None, Some(NonWorking(Break))]
    );
}

#[test]
fn periods_with_a_gap_between_them_are_not_warned() {
    assert_eq!(
        indicators(vec![
            period(900, Some(1200), Work),
            period(1230, Some(1300), Work),
        ]),
        vec![None, None]
    );
}

#[test]
fn a_needless_split_is_found_in_time_order_not_screen_order() {
    assert_eq!(
        indicators(vec![
            period(1200, Some(1300), Work),
            Period::default(),
            period(900, Some(1200), Work),
        ]),
        vec![Some(PeriodWarning), None, None]
    );
}

#[test]
fn a_non_working_period_shows_its_type() {
    assert_eq!(
        indicators(vec![
            period(900, Some(1200), Vacation),
            period(1230, Some(1300), Break),
        ]),
        vec![Some(NonWorking(Vacation)), Some(NonWorking(Break))]
    );
}

#[test]
fn a_period_warning_wins_over_the_period_type() {
    assert_eq!(
        indicators(vec![
            period(1200, Some(1230), Break),
            period(1230, Some(1300), Break),
        ]),
        vec![Some(NonWorking(Break)), Some(PeriodWarning)]
    );
}

#[test]
fn periods_involved_in_the_consistency_problem_are_marked_on_screen() {
    assert_eq!(
        indicators(vec![
            period(1130, Some(1300), Work),
            Period::default(),
            period(1400, Some(1500), Work),
            period(900, Some(1200), Work),
        ]),
        vec![
            Some(ConsistencyFailure),
            None,
            None,
            Some(ConsistencyFailure)
        ]
    );
}

#[test]
fn a_consistency_failure_wins_over_a_period_warning_and_the_type() {
    assert_eq!(
        indicators(vec![
            period(500, Some(1200), Break),
            period(1100, Some(1300), Work),
        ]),
        vec![Some(ConsistencyFailure), Some(ConsistencyFailure)]
    );
}

#[test]
fn a_zero_minute_period_is_a_consistency_failure() {
    assert_eq!(
        indicators(vec![period(1200, Some(1200), Work)]),
        vec![Some(ConsistencyFailure)]
    );
}

#[test]
fn only_the_current_problem_is_marked() {
    // Both periods lack a From; only the first problem counts.
    let no_from = |notes: &str| with_project_and_notes(Period::default(), None, Some(notes));
    assert_eq!(
        indicators(vec![no_from("a"), no_from("b")]),
        vec![Some(ConsistencyFailure), None]
    );
}

#[test]
fn an_empty_period_is_no_consistency_failure() {
    assert_eq!(
        indicators(vec![period(900, Some(1200), Work), Period::default()]),
        vec![None, None]
    );
}

#[test]
fn an_open_period_is_only_a_consistency_failure_on_another_day() {
    let d = day(vec![period(900, None, Work)]);
    assert_eq!(d.period_indicators(TODAY, TODAY), vec![None]);
    assert_eq!(
        d.period_indicators(PAST, TODAY),
        vec![Some(ConsistencyFailure)]
    );
}

#[test]
fn an_open_period_can_be_a_needless_split_too() {
    // Today, so the open period itself is allowed.
    let d = day(vec![
        period(900, Some(1200), Work),
        period(1200, None, Work),
    ]);
    assert_eq!(
        d.period_indicators(TODAY, TODAY),
        vec![None, Some(PeriodWarning)]
    );
}

#[test]
fn a_free_period_shows_its_type() {
    // Given
    let periods = vec![period(1300, Some(1500), Free)];

    // When
    let found = indicators(periods);

    // Then
    assert_eq!(found, vec![Some(NonWorking(Free))]);
}
