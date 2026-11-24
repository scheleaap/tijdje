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

/// A From–To pair with both times set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TimeSpan {
    from: TimeOfDay,
    to: TimeOfDay,
}

/// Where an auto-filled full day of absence starts.
const FULL_DAY_START: TimeOfDay = TimeOfDay::new(9, 0).expect("valid time");

/// 7h12m.
pub const DEFAULT_REGULAR_EXPECTED_WORKING_TIME: Minutes = Minutes(432);

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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Period {
    pub from: Option<TimeOfDay>,
    pub to: Option<TimeOfDay>,
    pub project: Option<String>,
    pub notes: Option<String>,
    pub period_type: PeriodType,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Day {
    pub regular_expected_working_time: Option<Minutes>,
    pub periods: Vec<Period>,
}

/// Stands in for days that aren't in the timesheet.
static EMPTY_DAY: Day = Day {
    regular_expected_working_time: None,
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

    pub fn is_empty(&self) -> bool {
        self.periods.iter().all(Period::is_empty)
    }

    pub fn worked_time(&self) -> Minutes {
        self.periods
            .iter()
            .filter(|p| p.period_type == PeriodType::Work)
            .filter_map(Period::duration)
            .sum()
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
            .filter_map(Period::duration)
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

    /// Adds an empty work period at the end whose From is the last period's To.
    /// Returns its index.
    pub fn add_period(&mut self) -> usize {
        let from = self.periods.last().and_then(|last| last.to);
        self.periods.push(Period {
            from,
            ..Period::default()
        });
        self.periods.len() - 1
    }

    /// Checks `period_type` on the period at `index`, or unchecks it (back to
    /// work) if it was already checked. Checking an absence on the day's only
    /// period, while its From and To are empty, fills a full day from 09:00.
    /// Unchecking an absence empties those times again, if they're unchanged.
    pub fn toggle_period_type(&mut self, index: usize, period_type: PeriodType) {
        let full_day = self.full_day();
        let is_only_period = self.periods.len() == 1;
        let Some(period) = self.periods.get_mut(index) else {
            return;
        };
        let previous = period.period_type;
        if previous == period_type {
            period.period_type = PeriodType::Work;
            if is_only_period && period_type.absence().is_some() {
                period.empty_times_if(full_day);
            }
        } else {
            period.period_type = period_type;
            let checking_an_absence =
                period_type.absence().is_some() && previous.absence().is_none();
            if is_only_period && checking_an_absence {
                period.fill_times_if_empty(full_day);
            }
        }
    }

    /// A full day of absence: from 09:00 for the regular expected working time.
    fn full_day(&self) -> Option<TimeSpan> {
        let end = FULL_DAY_START.minutes_since_midnight()
            + u16::try_from(self.regular_expected_working_time().0).ok()?;
        Some(TimeSpan {
            from: FULL_DAY_START,
            to: TimeOfDay::new(end / 60, end % 60)?,
        })
    }

    /// The first consistency problem of the day on `date`, if any.
    pub fn check(&self, date: Date, today: Date) -> Option<ConsistencyProblem> {
        self.check_from_is_set()
            .or_else(|| self.check_from_before_to())
            .or_else(|| self.check_no_overlaps())
            .or_else(|| self.check_open_period(date, today))
            .or_else(|| self.check_one_period_per_absence())
            .or_else(|| self.check_no_project_on_non_working_periods())
    }

    fn check_from_is_set(&self) -> Option<ConsistencyProblem> {
        self.first_period_where(
            |p| p.from.is_none(),
            |p| format!("Period {} has no From", p.label()),
        )
    }

    fn check_from_before_to(&self) -> Option<ConsistencyProblem> {
        self.first_period_where(
            |p| matches!((p.from, p.to), (Some(from), Some(to)) if from >= to),
            |p| format!("Period {}: From is not before To", p.label()),
        )
    }

    fn check_no_overlaps(&self) -> Option<ConsistencyProblem> {
        self.indices_by_from().windows(2).find_map(|pair| {
            let (earlier, later) = (&self.periods[pair[0]], &self.periods[pair[1]]);
            let message = match (earlier.to, later.from) {
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

    /// Period indices ordered by From; periods with the same From keep their order.
    fn indices_by_from(&self) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..self.periods.len()).collect();
        indices.sort_by_key(|&i| self.periods[i].from);
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

    /// Removes empty periods and sorts the rest by From. Returns, for each old
    /// period index, its new index (`None` if it was removed).
    pub fn normalize(&mut self) -> Vec<Option<usize>> {
        let kept: Vec<usize> = self
            .indices_by_from()
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

    /// Whether there is a gap between the period at `index` and the next one.
    pub fn has_implied_break_after(&self, index: usize) -> bool {
        self.implied_break_after(index).is_some()
    }

    /// The implied break after the period at `index`, as a break period from
    /// its end to the start of the next period, if there is a gap.
    pub fn implied_break_after(&self, index: usize) -> Option<Period> {
        let (period, next) = (self.periods.get(index)?, self.periods.get(index + 1)?);
        match (period.to, next.from) {
            (Some(to), Some(from)) if from > to => Some(Period {
                from: Some(to),
                to: Some(from),
                period_type: PeriodType::Break,
                ..Period::default()
            }),
            _ => None,
        }
    }

    pub fn day_balance(&self) -> Minutes {
        self.worked_time() - self.actual_expected_working_time()
    }
}

impl Period {
    /// An empty period has no values: no times, project or notes, and type work.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// The period's times as a label, e.g. `0900–1200`, `1230–` or `–1200`.
    pub fn label(&self) -> String {
        let time = |t: Option<TimeOfDay>| t.map(TimeOfDay::format_hhmm).unwrap_or_default();
        format!("{}–{}", time(self.from), time(self.to))
    }

    /// Sets From and To to `span` if both are empty.
    fn fill_times_if_empty(&mut self, span: Option<TimeSpan>) {
        if let (None, None, Some(span)) = (self.from, self.to, span) {
            self.from = Some(span.from);
            self.to = Some(span.to);
        }
    }

    /// Empties From and To if they hold exactly `span`.
    fn empty_times_if(&mut self, span: Option<TimeSpan>) {
        if span.is_some_and(|span| self.from == Some(span.from) && self.to == Some(span.to)) {
            self.from = None;
            self.to = None;
        }
    }

    /// An open period has no To yet, because it is still running.
    pub fn is_open(&self) -> bool {
        self.to.is_none()
    }

    /// Whether the period directly follows `previous` (its From is that
    /// period's To) with the same project, period type and notes.
    fn needlessly_splits(&self, previous: &Period) -> bool {
        previous.to.is_some()
            && previous.to == self.from
            && previous.project == self.project
            && previous.period_type == self.period_type
            && previous.notes == self.notes
    }

    /// Whether the period starts before 06:00 or ends after 20:00.
    pub fn has_unusual_time(&self) -> bool {
        self.from.is_some_and(|from| from < EARLIEST_USUAL_START)
            || self.to.is_some_and(|to| to > LATEST_USUAL_END)
    }

    /// The period's duration, or `None` while From or To is empty.
    pub fn duration(&self) -> Option<Minutes> {
        let (from, to) = (self.from?, self.to?);
        Some(to - from)
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
