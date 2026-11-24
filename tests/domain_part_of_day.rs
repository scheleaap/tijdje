mod common;

use common::{day, part_of_day_period, period, without_project};
use jiff::civil::{Date, date};
use tijdje::domain::{
    Extent, Minutes,
    PartOfDay::{FirstHalf, FullDay, SecondHalf},
    Period,
    PeriodType::{Break, Free, Holiday, Sick, Vacation, Work},
    TimeOfDay,
};

const TODAY: Date = date(2026, 11, 13);
const PAST: Date = date(2026, 11, 12);

fn time(hhmm: u16) -> Option<TimeOfDay> {
    TimeOfDay::new(hhmm / 100, hhmm % 100)
}

fn problem(periods: Vec<Period>) -> Option<(String, Vec<usize>)> {
    day(periods)
        .check(PAST, TODAY)
        .map(|p| (p.message, p.periods))
}

fn with_notes(mut p: Period, notes: &str) -> Period {
    p.notes = Some(notes.into());
    p
}

// Labels and durations

#[test]
fn parts_of_day_have_labels() {
    assert_eq!(FullDay.label(), "Full day");
    assert_eq!(FirstHalf.label(), "1st half");
    assert_eq!(SecondHalf.label(), "2nd half");
}

#[test]
fn a_part_of_day_period_is_named_by_its_label() {
    assert_eq!(part_of_day_period(FirstHalf, Work).label(), "1st half");
}

#[test]
fn a_full_day_lasts_the_regular_expected_working_time() {
    // Given
    let p = part_of_day_period(FullDay, Work);

    // When
    let duration = p.duration(Minutes(432));

    // Then
    assert_eq!(duration, Some(Minutes(432)));
}

#[test]
fn the_halves_add_up_to_a_full_day_with_the_first_rounded_down() {
    for (regular, first, second) in [(432, 216, 216), (431, 215, 216)] {
        // Given
        let regular = Minutes(regular);

        // When
        let durations = (
            part_of_day_period(FirstHalf, Work).duration(regular),
            part_of_day_period(SecondHalf, Work).duration(regular),
        );

        // Then
        assert_eq!(
            durations,
            (Some(Minutes(first)), Some(Minutes(second))),
            "{regular:?}"
        );
    }
}

#[test]
fn a_part_of_day_counts_as_worked_time() {
    // Given
    let d = day(vec![
        part_of_day_period(FirstHalf, Work),
        period(1300, Some(1400), Work),
    ]);

    // When
    let worked = d.worked_time();

    // Then: 3h36m plus 1h.
    assert_eq!(worked, Minutes(276));
}

#[test]
fn a_part_of_day_absence_reduces_the_actual_expected_working_time() {
    // Given
    let d = day(vec![
        part_of_day_period(FirstHalf, Vacation),
        part_of_day_period(SecondHalf, Work),
    ]);

    // When
    let figures = (d.actual_expected_working_time(), d.day_balance());

    // Then
    assert_eq!(figures, (Minutes(216), Minutes(0)));
}

#[test]
fn a_full_day_follows_the_days_regular_expected_working_time() {
    // Given
    let mut d = day(vec![part_of_day_period(FullDay, Sick)]);
    d.regular_expected_working_time = Some(Minutes(240));

    // When
    let absence = d.absence_time();

    // Then
    assert_eq!(absence, Minutes(240));
}

#[test]
fn a_part_of_day_period_is_never_open() {
    assert!(!part_of_day_period(FullDay, Work).is_open());
}

#[test]
fn a_part_of_day_period_is_not_empty() {
    assert!(!part_of_day_period(FullDay, Work).is_empty());
}

// Switching between from–to and part of day

#[test]
fn setting_a_part_of_day_empties_the_times() {
    // Given
    let mut d = day(vec![period(900, Some(1200), Work)]);

    // When
    d.switch_extent(0, Extent::PartOfDay(SecondHalf));

    // Then
    assert_eq!(d.periods[0], part_of_day_period(SecondHalf, Work));
}

#[test]
fn removing_the_part_of_day_leaves_empty_times() {
    // Given
    let mut d = day(vec![with_notes(part_of_day_period(FullDay, Free), "x")]);

    // When
    d.switch_extent(0, Extent::default());

    // Then
    assert_eq!(
        d.periods[0],
        with_notes(
            Period {
                period_type: Free,
                ..Period::default()
            },
            "x"
        )
    );
}

#[test]
fn removing_the_part_of_day_of_a_from_to_period_keeps_its_times() {
    // Given
    let mut d = day(vec![period(900, Some(1200), Work)]);

    // When
    d.switch_extent(0, Extent::default());

    // Then
    assert_eq!(d.periods[0], period(900, Some(1200), Work));
}

// Sorting

#[test]
fn sorting_puts_full_day_first_half_from_to_periods_and_second_half_in_order() {
    // Given
    let mut d = day(vec![
        part_of_day_period(SecondHalf, Work),
        period(1300, Some(1400), Work),
        part_of_day_period(FullDay, Holiday),
        Period::default(),
        period(900, Some(1000), Work),
        part_of_day_period(FirstHalf, Vacation),
    ]);

    // When
    let moved = d.normalize();

    // Then
    assert_eq!(
        d.periods,
        vec![
            part_of_day_period(FullDay, Holiday),
            part_of_day_period(FirstHalf, Vacation),
            period(900, Some(1000), Work),
            period(1300, Some(1400), Work),
            part_of_day_period(SecondHalf, Work),
        ]
    );
    assert_eq!(
        moved,
        vec![Some(4), Some(3), Some(0), None, Some(2), Some(1)]
    );
}

// Consistency checks

#[test]
fn a_part_of_day_period_needs_no_from() {
    assert_eq!(problem(vec![part_of_day_period(FullDay, Vacation)]), None);
}

#[test]
fn part_of_day_periods_never_overlap_from_to_periods() {
    // Given: a full day and a period during it, as on a holiday with some work.
    let periods = vec![
        part_of_day_period(FirstHalf, Holiday),
        period(900, Some(1000), Work),
        part_of_day_period(SecondHalf, Free),
    ];

    // When
    let found = problem(periods);

    // Then
    assert_eq!(found, None);
}

#[test]
fn a_part_of_day_is_allowed_for_every_type_but_break() {
    for period_type in [Work, Free, Vacation, Sick, Holiday] {
        // Given
        let periods = vec![part_of_day_period(FullDay, period_type)];

        // When
        let found = problem(periods);

        // Then
        assert_eq!(found, None, "{period_type:?}");
    }
}

#[test]
fn a_break_cannot_be_a_part_of_day() {
    assert_eq!(
        problem(vec![
            period(900, Some(1000), Work),
            part_of_day_period(FullDay, Break)
        ]),
        Some(("A break can't be a part of day (Full day)".into(), vec![1]))
    );
}

#[test]
fn at_most_one_full_day() {
    assert_eq!(
        problem(vec![
            part_of_day_period(FullDay, Vacation),
            part_of_day_period(FullDay, Work)
        ]),
        Some(("More than one full day".into(), vec![0, 1]))
    );
}

#[test]
fn at_most_one_of_each_half() {
    for (part, message) in [
        (FirstHalf, "More than one 1st half"),
        (SecondHalf, "More than one 2nd half"),
    ] {
        // Given
        let periods = vec![
            part_of_day_period(part, Vacation),
            period(900, Some(1000), Work),
            part_of_day_period(part, Work),
        ];

        // When
        let found = problem(periods);

        // Then
        assert_eq!(found, Some((message.into(), vec![0, 2])), "{part:?}");
    }
}

#[test]
fn a_full_day_never_goes_together_with_a_half() {
    assert_eq!(
        problem(vec![
            part_of_day_period(SecondHalf, Work),
            part_of_day_period(FullDay, Vacation)
        ]),
        Some(("Full day together with a half".into(), vec![0, 1]))
    );
}

#[test]
fn the_part_of_day_checks_come_after_the_existing_ones() {
    // Given: two full days, and one of them is a break with a project.
    let mut a_break = part_of_day_period(FullDay, Break);
    a_break.project = Some("acme".into());
    let periods = vec![a_break, part_of_day_period(FullDay, Work)];

    // When
    let found = problem(periods);

    // Then
    assert_eq!(
        found,
        Some(("Non-working period Full day has a project".into(), vec![0]))
    );
}

#[test]
fn the_break_check_comes_before_the_more_than_one_checks() {
    assert_eq!(
        problem(vec![
            part_of_day_period(FullDay, Work),
            part_of_day_period(FullDay, Break)
        ]),
        Some(("A break can't be a part of day (Full day)".into(), vec![1]))
    );
}

#[test]
fn a_period_without_from_or_part_of_day_fails_the_check() {
    assert_eq!(
        problem(vec![Period {
            project: Some("acme".into()),
            ..Period::default()
        }]),
        Some(("Period – has no From or part of day".into(), vec![0]))
    );
}

#[test]
fn no_implied_break_next_to_a_part_of_day_period() {
    // Given
    let d = day(vec![
        part_of_day_period(FirstHalf, Work),
        period(1300, Some(1400), Work),
    ]);

    // When
    let implied = d.has_implied_break_after(0);

    // Then
    assert!(!implied);
}

#[test]
fn an_implied_break_skips_part_of_day_and_empty_periods_in_between() {
    // Given
    let d = day(vec![
        period(900, Some(1000), Work),
        part_of_day_period(FullDay, Holiday),
        Period::default(),
        period(1100, Some(1200), Work),
    ]);

    // When
    let implied = d.implied_break_after(0);

    // Then
    assert_eq!(implied, Some(period(1000, Some(1100), Break)));
}

#[test]
fn no_implied_break_after_a_part_of_day_period_between_from_to_periods() {
    // Given
    let d = day(vec![
        period(900, Some(1000), Work),
        part_of_day_period(FullDay, Holiday),
        period(1100, Some(1200), Work),
    ]);

    // When
    let implied = d.has_implied_break_after(1);

    // Then
    assert!(!implied);
}

#[test]
fn a_part_of_day_may_follow_an_open_period_today() {
    // Given
    let d = day(vec![
        period(1300, None, Work),
        part_of_day_period(SecondHalf, Vacation),
    ]);

    // When
    let found = d.check(TODAY, TODAY);

    // Then
    assert_eq!(found, None);
}

// Checking and unchecking absences

#[test]
fn checking_an_absence_on_the_only_period_without_times_sets_a_full_day() {
    for absence in [Vacation, Sick, Holiday] {
        // Given
        let mut d = day(vec![with_notes(Period::default(), "Ski trip")]);

        // When
        d.toggle_period_type(0, absence);

        // Then
        let expected = with_notes(part_of_day_period(FullDay, absence), "Ski trip");
        assert_eq!(d.periods[0], expected, "{absence:?}");
    }
}

#[test]
fn checking_an_absence_never_fills_in_times() {
    // Given
    let mut d = day(vec![without_project(period(800, None, Work))]);

    // When
    d.toggle_period_type(0, Vacation);

    // Then
    assert_eq!(d.periods[0], period(800, None, Vacation));
}

#[test]
fn checking_an_absence_keeps_a_half() {
    // Given
    let mut d = day(vec![without_project(part_of_day_period(FirstHalf, Work))]);

    // When
    d.toggle_period_type(0, Sick);

    // Then
    assert_eq!(d.periods[0], part_of_day_period(FirstHalf, Sick));
}

#[test]
fn no_full_day_when_the_day_has_more_periods() {
    // Given
    let mut d = day(vec![Period::default(), period(900, Some(1000), Work)]);

    // When
    d.toggle_period_type(0, Vacation);

    // Then
    assert_eq!(d.periods[0].part_of_day(), None);
}

#[test]
fn checking_break_or_free_sets_no_full_day() {
    for period_type in [Break, Free] {
        // Given
        let mut d = day(vec![Period::default()]);

        // When
        d.toggle_period_type(0, period_type);

        // Then
        assert_eq!(d.periods[0].part_of_day(), None, "{period_type:?}");
    }
}

#[test]
fn checking_an_absence_on_a_period_with_a_project_sets_no_full_day() {
    // Given
    let mut d = day(vec![Period {
        project: Some("acme".into()),
        ..Period::default()
    }]);

    // When
    d.toggle_period_type(0, Vacation);

    // Then: checked, but without a full day.
    assert_eq!(d.periods[0].period_type, Vacation);
    assert_eq!(d.periods[0].extent, Extent::default());
}

#[test]
fn unchecking_the_absence_removes_the_full_day_and_keeps_the_notes() {
    // Given
    let mut d = day(vec![with_notes(Period::default(), "Ski trip")]);
    d.toggle_period_type(0, Vacation);

    // When
    d.toggle_period_type(0, Vacation);

    // Then
    assert_eq!(d.periods[0], with_notes(Period::default(), "Ski trip"));
}

#[test]
fn unchecking_keeps_a_half() {
    // Given
    let mut d = day(vec![part_of_day_period(SecondHalf, Vacation)]);

    // When
    d.toggle_period_type(0, Vacation);

    // Then
    assert_eq!(
        d.periods[0],
        without_project(part_of_day_period(SecondHalf, Work))
    );
}

#[test]
fn unchecking_keeps_the_full_day_when_the_day_has_more_periods() {
    // Given
    let mut d = day(vec![
        part_of_day_period(FullDay, Vacation),
        period(900, Some(1000), Work),
    ]);

    // When
    d.toggle_period_type(0, Vacation);

    // Then
    assert_eq!(
        d.periods[0],
        without_project(part_of_day_period(FullDay, Work))
    );
}

#[test]
fn unchecking_never_clears_times() {
    // Given
    let mut d = day(vec![period(900, Some(1612), Vacation)]);

    // When
    d.toggle_period_type(0, Vacation);

    // Then
    assert_eq!(d.periods[0], without_project(period(900, Some(1612), Work)));
}

#[test]
fn switching_between_absences_keeps_the_full_day() {
    // Given
    let mut d = day(vec![Period::default()]);
    d.toggle_period_type(0, Vacation);

    // When
    d.toggle_period_type(0, Sick);

    // Then
    assert_eq!(d.periods[0], part_of_day_period(FullDay, Sick));
}

// Chained times

#[test]
fn chaining_skips_part_of_day_and_empty_periods() {
    // Given
    let mut d = day(vec![
        period(900, Some(1200), Work),
        part_of_day_period(SecondHalf, Work),
        Period::default(),
        period(1200, Some(1300), Work),
    ]);

    // When
    d.set_to(0, time(1215));

    // Then
    assert_eq!(d.periods[3].from(), time(1215));
    assert_eq!(d.periods[1], part_of_day_period(SecondHalf, Work));
    assert_eq!(d.periods[2], Period::default());
}

#[test]
fn chaining_stops_at_the_next_from_to_period() {
    // Given
    let mut d = day(vec![
        period(900, Some(1200), Work),
        period(1230, Some(1300), Work),
        period(1200, Some(1400), Work),
    ]);

    // When
    d.set_to(0, time(1215));

    // Then
    assert_eq!(d.periods[2].from(), time(1200));
}
