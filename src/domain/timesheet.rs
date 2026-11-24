use std::collections::BTreeMap;

use jiff::civil::Date;

use super::{Minutes, TimeOfDay, calendar};

/// Periods starting before this time have an unusual time.
const EARLIEST_USUAL_START: TimeOfDay = TimeOfDay::new(6, 0).expect("valid time");
/// Periods ending after this time have an unusual time.
const LATEST_USUAL_END: TimeOfDay = TimeOfDay::new(20, 0).expect("valid time");

/// Why a day fails the consistency check, and which periods are involved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsistencyProblem {
    pub message: String,
    /// Indices of the periods involved.
    pub periods: Vec<usize>,
}

/// What the indicator left of From shows for a period.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodIndicator {
    /// The period is involved in the day's current consistency problem.
    ConsistencyFailure,
    /// The period is allowed but likely a mistake.
    PeriodWarning,
    /// The period is a non-working period of this type.
    NonWorking(PeriodType),
}

/// A day that fails the consistency check, with its problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InconsistentDay {
    pub date: Date,
    pub problem: ConsistencyProblem,
}

/// 7h12m.
pub const DEFAULT_REGULAR_EXPECTED_WORKING_TIME: Minutes = Minutes(432);
/// The worked time "Work until" aims for: 8h, fixed for now.
pub const WORK_UNTIL_TARGET: Minutes = Minutes(480);
/// The break time below which "Work until" adds the missing break time.
const MINIMUM_BREAK_TIME: Minutes = Minutes(60);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PeriodType {
    #[default]
    Work,
    Break,
    /// Time taken off from overtime: not worked time, and not an absence.
    Free,
    Vacation,
    Sick,
    Holiday,
}

/// A vacation, sick or holiday period's kind, in the priority shown in the left panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Absence {
    Sick,
    Vacation,
    Holiday,
}

impl From<Absence> for PeriodType {
    fn from(absence: Absence) -> Self {
        match absence {
            Absence::Sick => Self::Sick,
            Absence::Vacation => Self::Vacation,
            Absence::Holiday => Self::Holiday,
        }
    }
}

impl Absence {
    /// The absence's name in messages, e.g. "More than one sick period".
    pub fn name(self) -> &'static str {
        match self {
            Absence::Sick => "sick",
            Absence::Vacation => "vacation",
            Absence::Holiday => "holiday",
        }
    }
}

impl PeriodType {
    /// Whether this is a work period (as opposed to a non-working period).
    pub fn is_working(self) -> bool {
        self == PeriodType::Work
    }

    /// The absence this period type stands for, if any.
    pub fn absence(self) -> Option<Absence> {
        match self {
            Self::Sick => Some(Absence::Sick),
            Self::Vacation => Some(Absence::Vacation),
            Self::Holiday => Some(Absence::Holiday),
            Self::Work | Self::Break | Self::Free => None,
        }
    }
}

/// A period's extent as a share of the regular expected working time,
/// instead of From and To.
/// Ordered as sorted within a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PartOfDay {
    FullDay,
    FirstHalf,
    SecondHalf,
}

impl PartOfDay {
    /// All parts of day, in the order of their Alt+1/2/3 keys.
    pub const ALL: [PartOfDay; 3] = [Self::FullDay, Self::FirstHalf, Self::SecondHalf];

    /// The label shown instead of From and To, and used in messages.
    pub fn label(self) -> &'static str {
        match self {
            Self::FullDay => "Full day",
            Self::FirstHalf => "1st half",
            Self::SecondHalf => "2nd half",
        }
    }

    /// The duration on a day with `regular` as its regular expected working
    /// time: all of it for a full day; half of it, rounded down to the
    /// minute, for the 1st half, and the rest for the 2nd half, so both
    /// halves add up to a full day.
    pub fn duration(self, regular: Minutes) -> Minutes {
        let first_half = Minutes(regular.0.div_euclid(2));
        match self {
            Self::FullDay => regular,
            Self::FirstHalf => first_half,
            Self::SecondHalf => regular - first_half,
        }
    }

    /// Where a period with this part of day goes when its day is sorted: the
    /// 2nd half after the from–to periods, the others before them.
    fn sort_key(self) -> SortKey {
        match self {
            Self::SecondHalf => SortKey::Trailing(self),
            Self::FullDay | Self::FirstHalf => SortKey::Leading(self),
        }
    }
}

/// Where a period goes when a day is sorted: full day, 1st half, from–to
/// periods by From, 2nd half.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum SortKey {
    Leading(PartOfDay),
    FromToPeriod(Option<TimeOfDay>),
    Trailing(PartOfDay),
}

/// How a period spans its day: from a start time to an end time, or as a
/// part of day. A period never has both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extent {
    /// From and To, either of which may still be empty.
    FromTo {
        from: Option<TimeOfDay>,
        to: Option<TimeOfDay>,
    },
    PartOfDay(PartOfDay),
}

impl Default for Extent {
    /// From–to with empty times.
    fn default() -> Self {
        Self::FromTo {
            from: None,
            to: None,
        }
    }
}

/// Whether a period has From and To or a part of day, regardless of values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExtentKind {
    #[default]
    FromTo,
    PartOfDay,
}

impl Extent {
    /// Whether this is from–to or a part of day.
    pub fn kind(self) -> ExtentKind {
        match self {
            Self::FromTo { .. } => ExtentKind::FromTo,
            Self::PartOfDay(_) => ExtentKind::PartOfDay,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Period {
    pub extent: Extent,
    pub project: Option<String>,
    pub notes: Option<String>,
    pub period_type: PeriodType,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Day {
    pub regular_expected_working_time: Option<Minutes>,
    /// The day note: a remark attached to the day as a whole.
    pub day_note: Option<String>,
    pub periods: Vec<Period>,
}

/// Stands in for days that aren't in the timesheet.
static EMPTY_DAY: Day = Day {
    regular_expected_working_time: None,
    day_note: None,
    periods: Vec::new(),
};

/// A day sorted and without empty periods, for checks that don't depend on
/// the order on screen.
struct SortedDay {
    day: Day,
    /// For each period of `day`, the index of the same period on screen.
    on_screen: Vec<usize>,
}

impl SortedDay {
    /// The day's consistency problem, with the periods involved as indices
    /// on screen.
    fn consistency_problem(&self, date: Date, today: Date) -> Option<ConsistencyProblem> {
        let mut problem = self.day.check(date, today)?;
        for index in &mut problem.periods {
            *index = self.on_screen[*index];
        }
        Some(problem)
    }

    /// The base of "Work until": the latest To of the from–to periods, or
    /// the From of an open last period.
    fn work_until_base(&self) -> Option<TimeOfDay> {
        let from_to: Vec<&Period> = self
            .day
            .periods
            .iter()
            .filter(|p| p.from().is_some() || p.to().is_some())
            .collect();
        match from_to.last() {
            Some(last) if last.is_open() => last.from(),
            _ => from_to.iter().filter_map(|p| p.to()).max(),
        }
    }

    /// The minimum break time minus the explicit and implied break time, if
    /// that is positive. Free periods don't count.
    fn missing_break_time(&self) -> Minutes {
        let day = &self.day;
        let explicit: Minutes = day
            .periods
            .iter()
            .filter(|p| p.period_type == PeriodType::Break)
            .filter_map(|p| day.duration_of(p))
            .sum();
        let implied: Minutes = (0..day.periods.len())
            .filter_map(|index| day.implied_break_after(index))
            .filter_map(|p| day.duration_of(&p))
            .sum();
        (MINIMUM_BREAK_TIME - explicit - implied).max(Minutes(0))
    }
}

impl Day {
    /// A day is empty when it has no periods other than empty periods.
    /// Whether the day has no periods at all; the right panel then shows one
    /// empty period, which only becomes a real period once something is
    /// filled in.
    pub fn shows_empty_period(&self) -> bool {
        self.periods.is_empty()
    }

    /// The day as the right panel shows it: with one empty period if it has
    /// no periods.
    pub fn as_shown(&self) -> std::borrow::Cow<'_, Day> {
        if self.shows_empty_period() {
            let mut day = self.clone();
            day.periods.push(Period::default());
            std::borrow::Cow::Owned(day)
        } else {
            std::borrow::Cow::Borrowed(self)
        }
    }

    /// An empty day has no periods other than empty periods. Its day note
    /// doesn't count: a day with only a day note is empty too.
    pub fn is_empty(&self) -> bool {
        self.periods.iter().all(Period::is_empty)
    }

    /// Sets the day note to `text`; empty text or only spaces is no day note.
    pub fn set_day_note(&mut self, text: &str) {
        self.day_note = (!text.trim().is_empty()).then(|| text.to_owned());
    }

    /// Whether the day has a day note or a period that isn't empty; only
    /// such days are saved.
    pub fn has_content(&self) -> bool {
        self.day_note.is_some() || !self.is_empty()
    }

    pub fn worked_time(&self) -> Minutes {
        self.periods
            .iter()
            .filter(|p| p.period_type == PeriodType::Work)
            .filter_map(|p| self.duration_of(p))
            .sum()
    }

    /// The duration of `period` on this day: a part of day depends on the
    /// day's regular expected working time.
    pub fn duration_of(&self, period: &Period) -> Option<Minutes> {
        period.duration(self.regular_expected_working_time())
    }

    /// The stored regular expected working time, or the default.
    pub fn regular_expected_working_time(&self) -> Minutes {
        self.regular_expected_working_time
            .unwrap_or(DEFAULT_REGULAR_EXPECTED_WORKING_TIME)
    }

    /// The total duration of the day's absences.
    pub fn absence_time(&self) -> Minutes {
        self.periods
            .iter()
            .filter(|p| p.period_type.absence().is_some())
            .filter_map(|p| self.duration_of(p))
            .sum()
    }

    pub fn actual_expected_working_time(&self) -> Minutes {
        self.regular_expected_working_time() - self.absence_time()
    }

    /// The absence shown for the day: sick, then vacation, then holiday.
    pub fn absence(&self) -> Option<Absence> {
        self.periods
            .iter()
            .filter_map(|p| p.period_type.absence())
            .min()
    }

    /// Adds an empty work period at the end whose From is the last period's
    /// To (empty after a part-of-day period). Returns its index.
    pub fn add_period(&mut self) -> usize {
        let from = self.periods.last().and_then(Period::to);
        self.periods.push(Period {
            extent: Extent::FromTo { from, to: None },
            ..Period::default()
        });
        self.periods.len() - 1
    }

    /// Sets the From of the period at `index`, if there is one, making it a
    /// from–to period. Changes nothing else.
    pub fn set_from(&mut self, index: usize, from: Option<TimeOfDay>) {
        if let Some(period) = self.periods.get_mut(index) {
            period.extent = Extent::FromTo {
                from,
                to: period.to(),
            };
        }
    }

    /// Sets the To of the period at `index`, if there is one, making it a
    /// from–to period. If the next from–to period's From was equal to the old
    /// To, it moves along, so consecutive periods stay connected; part-of-day
    /// and empty periods in between are skipped. Clearing a To, or filling in
    /// the To of an open period, leaves the next From alone.
    pub fn set_to(&mut self, index: usize, to: Option<TimeOfDay>) {
        let old_to = self.periods.get_mut(index).and_then(|period| {
            let old_to = period.to();
            period.extent = Extent::FromTo {
                from: period.from(),
                to,
            };
            old_to
        });
        let next = self
            .next_from_to_period(index)
            .and_then(|next| self.periods.get_mut(next));
        if let (Some(old_to), Some(_), Some(next)) = (old_to, to, next)
            && next.from() == Some(old_to)
        {
            next.extent = Extent::FromTo {
                from: to,
                to: next.to(),
            };
        }
    }

    /// The index of the first from–to period after the period at `index`
    /// that isn't empty.
    fn next_from_to_period(&self, index: usize) -> Option<usize> {
        (index + 1..self.periods.len()).find(|&i| {
            let period = &self.periods[i];
            period.extent.kind() == ExtentKind::FromTo && !period.is_empty()
        })
    }

    /// Switches the period at `index`, if there is one, to `extent`. A part
    /// of day replaces the times; switching to from–to only applies to a
    /// part-of-day period, and a from–to period keeps its times.
    pub fn switch_extent(&mut self, index: usize, extent: Extent) {
        if let Some(period) = self.periods.get_mut(index)
            && (extent.kind() == ExtentKind::PartOfDay
                || period.extent.kind() == ExtentKind::PartOfDay)
        {
            period.extent = extent;
        }
    }

    /// Checks `period_type` on the period at `index`, or unchecks it (back to
    /// work) if it was already checked. Checking an absence on the day's only
    /// period, while it has no times, part of day or project, makes it a full day.
    /// Unchecking an absence removes the full day while it is still the only
    /// period and still a full day, even one set by hand. Times are never
    /// filled in or cleared.
    pub fn toggle_period_type(&mut self, index: usize, period_type: PeriodType) {
        let is_only_period = self.periods.len() == 1;
        if let Some(period) = self.periods.get_mut(index) {
            let unchecking = period.period_type == period_type;
            let full_day = Extent::PartOfDay(PartOfDay::FullDay);
            period.period_type = if unchecking {
                PeriodType::Work
            } else {
                period_type
            };
            let full_day_applies = is_only_period && period_type.absence().is_some();
            if full_day_applies && unchecking && period.extent == full_day {
                period.extent = Extent::default();
            } else if full_day_applies && !unchecking && period.has_no_extent_or_project() {
                period.extent = full_day;
            }
        }
    }

    /// The first consistency problem of the day on `date`, if any.
    pub fn check(&self, date: Date, today: Date) -> Option<ConsistencyProblem> {
        self.check_from_is_set()
            .or_else(|| self.check_from_before_to())
            .or_else(|| self.check_no_overlaps())
            .or_else(|| self.check_open_period(date, today))
            .or_else(|| self.check_one_period_per_absence())
            .or_else(|| self.check_no_project_on_non_working_periods())
            .or_else(|| self.check_no_part_of_day_on_breaks())
            .or_else(|| {
                PartOfDay::ALL
                    .into_iter()
                    .find_map(|part| self.check_at_most_one(part))
            })
            .or_else(|| self.check_no_full_day_with_a_half())
    }

    fn check_from_is_set(&self) -> Option<ConsistencyProblem> {
        self.first_period_where(
            |p| matches!(p.extent, Extent::FromTo { from: None, .. }),
            |p| format!("Period {} has no From or part of day", p.label()),
        )
    }

    fn check_from_before_to(&self) -> Option<ConsistencyProblem> {
        self.first_period_where(
            |p| matches!((p.from(), p.to()), (Some(from), Some(to)) if from >= to),
            |p| format!("Period {}: From is not before To", p.label()),
        )
    }

    /// Only from–to periods can overlap.
    fn check_no_overlaps(&self) -> Option<ConsistencyProblem> {
        let from_to_periods: Vec<usize> = self
            .sorted_indices()
            .into_iter()
            .filter(|&i| self.periods[i].extent.kind() == ExtentKind::FromTo)
            .collect();
        from_to_periods.windows(2).find_map(|pair| {
            let (earlier, later) = (&self.periods[pair[0]], &self.periods[pair[1]]);
            let message = match (earlier.to(), later.from()) {
                (Some(to), Some(from)) if from < to => {
                    format!("{} overlaps {}", earlier.label(), later.label())
                }
                (None, Some(_)) => format!(
                    "Open period {} is followed by {}",
                    earlier.label(),
                    later.label()
                ),
                _ => return None,
            };
            Some(ConsistencyProblem {
                message,
                periods: pair.to_vec(),
            })
        })
    }

    fn check_open_period(&self, date: Date, today: Date) -> Option<ConsistencyProblem> {
        if date == today {
            return None;
        }
        let when = if date < today { "past" } else { "future" };
        self.first_period_where(Period::is_open, |p| {
            format!("Open period {} on a {when} day", p.label())
        })
    }

    fn check_one_period_per_absence(&self) -> Option<ConsistencyProblem> {
        [Absence::Sick, Absence::Vacation, Absence::Holiday]
            .into_iter()
            .find_map(|absence| {
                let periods: Vec<usize> = (0..self.periods.len())
                    .filter(|&i| self.periods[i].period_type.absence() == Some(absence))
                    .collect();
                (periods.len() > 1).then(|| ConsistencyProblem {
                    message: format!("More than one {} period", absence.name()),
                    periods,
                })
            })
    }

    fn check_no_project_on_non_working_periods(&self) -> Option<ConsistencyProblem> {
        self.first_period_where(
            |p| !p.period_type.is_working() && p.project.is_some(),
            |p| format!("Non-working period {} has a project", p.label()),
        )
    }

    fn check_no_part_of_day_on_breaks(&self) -> Option<ConsistencyProblem> {
        self.first_period_where(
            |p| p.period_type == PeriodType::Break && p.part_of_day().is_some(),
            |p| format!("A break can't be a part of day ({})", p.label()),
        )
    }

    /// A problem when more than one period has `part` as its part of day,
    /// e.g. "More than one 1st half".
    fn check_at_most_one(&self, part: PartOfDay) -> Option<ConsistencyProblem> {
        let periods = self.indices_where(|p| p.part_of_day() == Some(part));
        (periods.len() > 1).then(|| ConsistencyProblem {
            message: format!("More than one {}", part.label().to_lowercase()),
            periods,
        })
    }

    fn check_no_full_day_with_a_half(&self) -> Option<ConsistencyProblem> {
        let has_full_day = self
            .periods
            .iter()
            .any(|p| p.part_of_day() == Some(PartOfDay::FullDay));
        // The full day and the halves.
        let periods = self.indices_where(|p| p.part_of_day().is_some());
        (has_full_day && periods.len() > 1).then(|| ConsistencyProblem {
            message: "Full day together with a half".to_owned(),
            periods,
        })
    }

    /// The indices of the periods matching `matches`.
    fn indices_where(&self, matches: impl Fn(&Period) -> bool) -> Vec<usize> {
        (0..self.periods.len())
            .filter(|&i| matches(&self.periods[i]))
            .collect()
    }

    /// A problem for the first period matching `fails`, described by `message`.
    fn first_period_where(
        &self,
        fails: impl Fn(&Period) -> bool,
        message: impl Fn(&Period) -> String,
    ) -> Option<ConsistencyProblem> {
        let index = self.periods.iter().position(fails)?;
        Some(ConsistencyProblem {
            message: message(&self.periods[index]),
            periods: vec![index],
        })
    }

    /// Period indices in sort order (full day, 1st half, from–to periods by
    /// From, 2nd half); periods in the same place keep their order.
    fn sorted_indices(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.periods.len()).collect();
        indices.sort_by_key(|&i| self.periods[i].sort_key());
        indices
    }

    /// The indicator left of From for each period on screen: a consistency
    /// failure, else a period warning, else a non-working period's type.
    /// Determined live, on a sorted copy of the day without empty periods.
    pub fn period_indicators(&self, date: Date, today: Date) -> Vec<Option<PeriodIndicator>> {
        let mut indicators: Vec<Option<PeriodIndicator>> = self
            .periods
            .iter()
            .map(|p| {
                (!p.period_type.is_working()).then_some(PeriodIndicator::NonWorking(p.period_type))
            })
            .collect();
        let sorted = self.sorted_copy();
        for (index, period) in sorted.day.periods.iter().enumerate() {
            let previous = index.checked_sub(1).map(|i| &sorted.day.periods[i]);
            if period.has_unusual_time() || previous.is_some_and(|p| period.needlessly_splits(p)) {
                indicators[sorted.on_screen[index]] = Some(PeriodIndicator::PeriodWarning);
            }
        }
        if let Some(problem) = sorted.consistency_problem(date, today) {
            for index in problem.periods {
                indicators[index] = Some(PeriodIndicator::ConsistencyFailure);
            }
        }
        indicators
    }

    /// A sorted copy of the day without empty periods, remembering where
    /// each of its periods is on screen.
    fn sorted_copy(&self) -> SortedDay {
        let mut day = self.clone();
        let moved = day.normalize();
        let mut on_screen = vec![0; day.periods.len()];
        for (old, new) in moved.into_iter().enumerate() {
            if let Some(new) = new {
                on_screen[new] = old;
            }
        }
        SortedDay { day, on_screen }
    }

    /// Removes empty periods and sorts the rest: full day, 1st half, from–to
    /// periods by From, 2nd half. Returns, for each old period index, its new
    /// index (`None` if it was removed).
    pub fn normalize(&mut self) -> Vec<Option<usize>> {
        let kept: Vec<usize> = self
            .sorted_indices()
            .into_iter()
            .filter(|&i| !self.periods[i].is_empty())
            .collect();
        let mut moved = vec![None; self.periods.len()];
        for (new, &old) in kept.iter().enumerate() {
            moved[old] = Some(new);
        }
        let old = std::mem::take(&mut self.periods);
        let mut old: Vec<Option<Period>> = old.into_iter().map(Some).collect();
        self.periods = kept.iter().filter_map(|&i| old[i].take()).collect();
        moved
    }

    /// Removes the period at `index`, if there is one.
    pub fn remove_period(&mut self, index: usize) {
        if index < self.periods.len() {
            self.periods.remove(index);
        }
    }

    /// Whether there is a gap between the from–to period at `index` and the
    /// next from–to period.
    pub fn has_implied_break_after(&self, index: usize) -> bool {
        self.implied_break_after(index).is_some()
    }

    /// The implied break after the from–to period at `index`, as a break
    /// period from its end to the start of the next from–to period, if there
    /// is a gap. Part-of-day and empty periods in between are skipped.
    pub fn implied_break_after(&self, index: usize) -> Option<Period> {
        let period = self.periods.get(index)?;
        let next = &self.periods[self.next_from_to_period(index)?];
        match (period.to(), next.from()) {
            (Some(to), Some(from)) if from > to => Some(Period {
                extent: Extent::FromTo {
                    from: Some(to),
                    to: Some(from),
                },
                period_type: PeriodType::Break,
                ..Period::default()
            }),
            _ => None,
        }
    }

    pub fn day_balance(&self) -> Minutes {
        self.worked_time() - self.actual_expected_working_time()
    }

    /// The time until which to work to reach 8h worked, given the periods as
    /// they are: the base, plus the time still to work, plus the missing
    /// break time. `None` once 8h is worked, without from–to periods, or at
    /// 24:00 or later. Uses the periods in time order.
    pub fn work_until(&self) -> Option<TimeOfDay> {
        let remaining = WORK_UNTIL_TARGET - self.worked_time();
        let sorted = self.sorted_copy();
        (remaining > Minutes(0))
            .then(|| sorted.work_until_base())
            .flatten()
            .and_then(|base| base.checked_add(remaining + sorted.missing_break_time()))
    }
}

impl Period {
    /// An empty period has no values: no times, part of day, project or
    /// notes, and type work.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The period's times as a label, e.g. `0900–1200`, `1230–` or `–1200`;
    /// for a part-of-day period its part of day, e.g. `Full day`.
    pub fn label(&self) -> String {
        let time = |t: Option<TimeOfDay>| t.map(TimeOfDay::format_hhmm).unwrap_or_default();
        match self.extent {
            Extent::FromTo { from, to } => format!("{}–{}", time(from), time(to)),
            Extent::PartOfDay(part) => part.label().to_owned(),
        }
    }

    /// The period's From; `None` while empty, and for a part-of-day period.
    pub fn from(&self) -> Option<TimeOfDay> {
        match self.extent {
            Extent::FromTo { from, .. } => from,
            Extent::PartOfDay(_) => None,
        }
    }

    /// The period's To; `None` while empty, and for a part-of-day period.
    pub fn to(&self) -> Option<TimeOfDay> {
        match self.extent {
            Extent::FromTo { to, .. } => to,
            Extent::PartOfDay(_) => None,
        }
    }

    /// The period's part of day; `None` for a from–to period.
    pub fn part_of_day(&self) -> Option<PartOfDay> {
        match self.extent {
            Extent::FromTo { .. } => None,
            Extent::PartOfDay(part) => Some(part),
        }
    }

    /// Whether the period has no times, no part of day and no project; notes
    /// don't count. Only then may checking an absence make it a full day.
    fn has_no_extent_or_project(&self) -> bool {
        self.extent == Extent::default() && self.project.is_none()
    }

    /// Where the period goes when its day is sorted.
    fn sort_key(&self) -> SortKey {
        match self.extent {
            Extent::FromTo { from, .. } => SortKey::FromToPeriod(from),
            Extent::PartOfDay(part) => part.sort_key(),
        }
    }

    /// An open period has no To yet, because it is still running. A
    /// part-of-day period is never open.
    pub fn is_open(&self) -> bool {
        matches!(self.extent, Extent::FromTo { to: None, .. })
    }

    /// Whether the period directly follows `previous` (its From is that
    /// period's To) with the same project, period type and notes.
    fn needlessly_splits(&self, previous: &Period) -> bool {
        previous.to().is_some()
            && previous.to() == self.from()
            && previous.project == self.project
            && previous.period_type == self.period_type
            && previous.notes == self.notes
    }

    /// Whether the period starts before 06:00 or ends after 20:00.
    pub fn has_unusual_time(&self) -> bool {
        self.from().is_some_and(|from| from < EARLIEST_USUAL_START)
            || self.to().is_some_and(|to| to > LATEST_USUAL_END)
    }

    /// The period's duration on a day with `regular` as its regular expected
    /// working time, or `None` while From or To of a from–to period is empty.
    pub fn duration(&self, regular: Minutes) -> Option<Minutes> {
        match self.extent {
            Extent::FromTo { from, to } => Some(to? - from?),
            Extent::PartOfDay(part) => Some(part.duration(regular)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Timesheet {
    pub days: BTreeMap<Date, Day>,
}

impl Timesheet {
    /// The day on `date`; empty if it isn't in the timesheet.
    pub fn day(&self, date: Date) -> &Day {
        self.days.get(&date).unwrap_or(&EMPTY_DAY)
    }

    /// The day on `date` for changing it; added to the timesheet if it isn't there.
    pub fn day_mut(&mut self, date: Date) -> &mut Day {
        self.days.entry(date).or_default()
    }

    /// Replaces the day on `date`.
    pub fn set_day(&mut self, date: Date, day: Day) {
        self.days.insert(date, day);
    }

    /// The worked time on `date`, or `None` when nothing is recorded that day.
    pub fn worked_time_on(&self, date: Date) -> Option<Minutes> {
        let day = self.day(date);
        (!day.is_empty()).then(|| day.worked_time())
    }

    pub fn absence_on(&self, date: Date) -> Option<Absence> {
        self.day(date).absence()
    }

    /// The day balance on `date`, from the first recorded day up to and including `today`.
    pub fn day_balance_on(&self, date: Date, today: Date) -> Option<Minutes> {
        self.has_balances_on(date, today)
            .then(|| self.day(date).day_balance())
    }

    /// The sum of day balances from the first recorded day up to and including `date`.
    pub fn total_balance_on(&self, date: Date, today: Date) -> Option<Minutes> {
        if !self.has_balances_on(date, today) {
            return None;
        }
        let first = self.first_recorded_day()?;
        Some(
            std::iter::successors(Some(first), |&d| Some(calendar::next_weekday(d)))
                .take_while(|&d| d <= date)
                .map(|d| self.day(d).day_balance())
                .sum(),
        )
    }

    /// Whether a day fails the consistency check, or is an empty day before
    /// today from the first recorded day on.
    pub fn has_day_warning_on(&self, date: Date, today: Date) -> bool {
        let day = self.day(date);
        if day.is_empty() {
            self.is_tracked(date) && date < today
        } else {
            day.sorted_copy().consistency_problem(date, today).is_some()
        }
    }

    /// The earliest day that fails the consistency check, with its problem.
    pub fn first_inconsistent_day(&self, today: Date) -> Option<InconsistentDay> {
        self.days.iter().find_map(|(&date, day)| {
            day.check(date, today)
                .map(|problem| InconsistentDay { date, problem })
        })
    }

    /// The earliest day with at least one period that isn't empty.
    pub fn first_recorded_day(&self) -> Option<Date> {
        self.days
            .iter()
            .find(|(_, day)| !day.is_empty())
            .map(|(&date, _)| date)
    }

    /// The project to suggest while `typed` is being entered as a project:
    /// of the projects used in the timesheet that start with `typed` (ignoring
    /// case) and are longer, the most used one; ties go to the one used most
    /// recently. `None` for empty `typed`.
    pub fn project_suggestion(&self, typed: &str) -> Option<&str> {
        if typed.is_empty() {
            return None;
        }
        // Each use of a matching project, with its position among all uses.
        let mut uses: Vec<(&str, usize)> = self
            .days
            .values()
            .flat_map(|day| &day.periods)
            .filter_map(|period| period.project.as_deref())
            .enumerate()
            .filter(|&(_, project)| completes(project, typed))
            .map(|(position, project)| (project, position))
            .collect();
        uses.sort_unstable();
        uses.chunk_by(|a, b| a.0 == b.0)
            .map(|same_project| {
                let (project, last_use) = same_project[same_project.len() - 1];
                let usage = ProjectUsage {
                    count: same_project.len(),
                    last_use,
                };
                (project, usage)
            })
            .max_by_key(|&(_, usage)| usage)
            .map(|(project, _)| project)
    }

    /// Whether `date` is on or after the first recorded day.
    fn is_tracked(&self, date: Date) -> bool {
        self.first_recorded_day().is_some_and(|first| date >= first)
    }

    /// Balances are shown from the first recorded day up to and including today.
    fn has_balances_on(&self, date: Date, today: Date) -> bool {
        self.is_tracked(date) && date <= today
    }
}

/// How a project is used, ordered from least to most suited as a suggestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct ProjectUsage {
    /// The number of periods with the project.
    count: usize,
    /// The position of the last period with the project, in date order.
    last_use: usize,
}

/// Whether `project` starts with `typed`, ignoring case, and is longer.
fn completes(project: &str, typed: &str) -> bool {
    project.chars().count() > typed.chars().count()
        && project
            .chars()
            .zip(typed.chars())
            .all(|(p, t)| p.to_lowercase().eq(t.to_lowercase()))
}
