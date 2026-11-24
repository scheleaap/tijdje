mod common;

use std::collections::BTreeMap;

use common::{PROJECT, day, part_of_day_period, period, without_project};
use jiff::civil::{Date, date};
use tijdje::domain::{
    Extent,
    PartOfDay::FullDay,
    Period,
    PeriodType::{Break, Free, Holiday, Sick, Vacation, Work},
    Timesheet,
};

const TODAY: Date = date(2026, 11, 13);
const PAST: Date = date(2026, 11, 12);

#[test]
fn normalizing_removes_empty_periods_and_sorts_by_from() {
    let mut d = day(vec![
        period(1300, Some(1700), Work),
        Period::default(),
        period(900, Some(1200), Work),
    ]);
    let moved = d.normalize();
    assert_eq!(
        d.periods,
        vec![
            period(900, Some(1200), Work),
            period(1300, Some(1700), Work)
        ]
    );
    // Where each old period went: the first moved to 1, the empty one was removed.
    assert_eq!(moved, vec![Some(1), None, Some(0)]);
}

#[test]
fn normalizing_a_sorted_day_changes_nothing() {
    let periods = vec![
        period(900, Some(1200), Work),
        period(1300, Some(1700), Work),
    ];
    let mut d = day(periods.clone());
    assert_eq!(d.normalize(), vec![Some(0), Some(1)]);
    assert_eq!(d.periods, periods);
}

fn problem(periods: Vec<Period>, date: Date) -> Option<(String, Vec<usize>)> {
    day(periods)
        .check(date, TODAY)
        .map(|p| (p.message, p.periods))
}

fn with_project(mut p: Period) -> Period {
    p.project = Some(PROJECT.into());
    p
}

#[test]
fn a_consistent_day_passes() {
    assert_eq!(
        problem(
            vec![
                period(900, Some(1200), Work),
                period(1200, Some(1230), Break),
                period(1300, Some(1500), Vacation),
                period(1500, Some(1600), Sick),
                period(1600, Some(1700), Holiday),
            ],
            PAST
        ),
        None
    );
}

#[test]
fn non_consecutive_periods_are_allowed() {
    assert_eq!(
        problem(
            vec![
                period(900, Some(1200), Work),
                period(1300, Some(1700), Work)
            ],
            PAST
        ),
        None
    );
}

#[test]
fn a_period_needs_a_from() {
    let mut p = period(900, Some(1200), Work);
    p.extent = Extent::FromTo {
        from: None,
        to: p.to(),
    };
    assert_eq!(
        problem(vec![p], PAST),
        Some(("Period –1200 has no From or part of day".into(), vec![0]))
    );
}

#[test]
fn from_must_be_before_to() {
    assert_eq!(
        problem(vec![period(1300, Some(1200), Work)], PAST),
        Some(("Period 1300–1200: From is not before To".into(), vec![0]))
    );
    assert!(problem(vec![period(1200, Some(1200), Work)], PAST).is_some());
}

#[test]
fn periods_must_not_overlap() {
    assert_eq!(
        problem(
            vec![
                period(900, Some(1200), Work),
                period(1130, Some(1300), Work)
            ],
            PAST
        ),
        Some(("0900–1200 overlaps 1130–1300".into(), vec![0, 1]))
    );
}

#[test]
fn overlaps_are_found_in_unsorted_days_too() {
    assert_eq!(
        problem(
            vec![
                period(1130, Some(1300), Work),
                period(900, Some(1200), Work)
            ],
            PAST
        ),
        Some(("0900–1200 overlaps 1130–1300".into(), vec![1, 0]))
    );
}

#[test]
fn an_open_period_is_only_allowed_today() {
    assert_eq!(problem(vec![period(1230, None, Work)], TODAY), None);
    assert_eq!(
        problem(vec![period(1230, None, Work)], PAST),
        Some(("Open period 1230– on a past day".into(), vec![0]))
    );
    assert_eq!(
        problem(vec![period(1230, None, Work)], date(2026, 11, 16)),
        Some(("Open period 1230– on a future day".into(), vec![0]))
    );
}

#[test]
fn at_most_one_period_per_absence() {
    for (absence, name) in [(Sick, "sick"), (Vacation, "vacation"), (Holiday, "holiday")] {
        assert_eq!(
            problem(
                vec![
                    period(900, Some(1000), absence),
                    period(1100, Some(1200), absence)
                ],
                PAST
            ),
            Some((format!("More than one {name} period"), vec![0, 1]))
        );
    }
}

#[test]
fn a_day_may_have_more_than_one_free_period() {
    // Given
    let periods = vec![
        period(900, Some(1000), Free),
        period(1100, Some(1200), Free),
    ];

    // When
    let found = problem(periods, PAST);

    // Then
    assert_eq!(found, None);
}

#[test]
fn non_working_periods_have_no_project() {
    assert_eq!(problem(vec![period(900, Some(1200), Work)], PAST), None);
    assert_eq!(
        problem(vec![with_project(period(1200, Some(1230), Break))], PAST),
        Some(("Non-working period 1200–1230 has a project".into(), vec![0]))
    );
}

#[test]
fn a_free_period_has_no_project() {
    // Given
    let periods = vec![with_project(period(1300, Some(1500), Free))];

    // When
    let found = problem(periods, PAST);

    // Then
    assert_eq!(
        found,
        Some(("Non-working period 1300–1500 has a project".into(), vec![0]))
    );
}

#[test]
fn a_day_with_a_problem_gets_a_day_warning() {
    let t = Timesheet {
        days: BTreeMap::from([
            (date(2026, 11, 12), day(vec![period(900, Some(1700), Work)])),
            (TODAY, day(vec![period(1300, Some(1200), Work)])),
        ]),
    };
    assert!(
        t.has_day_warning_on(TODAY, TODAY),
        "inconsistent days get a warning, even today"
    );
    assert!(!t.has_day_warning_on(PAST, TODAY));
}

#[test]
fn the_earliest_inconsistent_day_is_found() {
    let t = Timesheet {
        days: BTreeMap::from([
            (date(2026, 11, 10), day(vec![period(900, Some(1700), Work)])),
            (
                date(2026, 11, 11),
                day(vec![period(1300, Some(1200), Work)]),
            ),
            (date(2026, 11, 12), day(vec![period(1230, None, Work)])),
        ]),
    };
    let found = t.first_inconsistent_day(TODAY).unwrap();
    assert_eq!(found.date, date(2026, 11, 11));
    assert_eq!(found.problem.periods, vec![0]);
}

#[test]
fn nothing_may_follow_an_open_period() {
    assert_eq!(
        problem(
            vec![period(900, None, Work), period(1000, Some(1100), Work)],
            TODAY
        ),
        Some((
            "Open period 0900– is followed by 1000–1100".into(),
            vec![0, 1]
        ))
    );
}

#[test]
fn a_work_period_needs_a_project() {
    // Given
    let periods = vec![
        period(900, Some(1200), Work),
        without_project(period(1300, Some(1700), Work)),
    ];

    // When
    let found = problem(periods, PAST);

    // Then
    assert_eq!(
        found,
        Some(("Work period 1300–1700 has no project".into(), vec![1]))
    );
}

#[test]
fn an_open_work_period_today_needs_a_project_too() {
    // Given
    let periods = vec![without_project(period(900, None, Work))];

    // When
    let found = problem(periods, TODAY);

    // Then
    assert_eq!(
        found,
        Some(("Work period 0900– has no project".into(), vec![0]))
    );
}

#[test]
fn a_part_of_day_work_period_needs_a_project() {
    // Given
    let periods = vec![without_project(part_of_day_period(FullDay, Work))];

    // When
    let found = problem(periods, PAST);

    // Then
    assert_eq!(
        found,
        Some(("Work period Full day has no project".into(), vec![0]))
    );
}

#[test]
fn non_working_periods_without_a_project_pass() {
    // Given
    let periods = vec![
        period(900, Some(1000), Break),
        period(1000, Some(1100), Free),
        period(1100, Some(1200), Vacation),
        period(1300, Some(1400), Sick),
        period(1400, Some(1500), Holiday),
    ];

    // When
    let found = problem(periods, PAST);

    // Then
    assert_eq!(found, None);
}
