use std::collections::BTreeMap;

use jiff::civil::Date;

use super::{Minutes, TimeOfDay, calendar};

/// Periods starting before this time have an unusual time.
const EARLIEST_USUAL_START: TimeOfDay = TimeOfDay::new(6, 0).expect("valid time");
/// Periods ending after this time have an unusual time.
const LATEST_USUAL_END: TimeOfDay = TimeOfDay::new(20, 0).expect("valid time");

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

impl PeriodType {
    /// The absence this period type stands for, if any.
    pub fn absence(self) -> Option<Absence> {
        match self {
            Self::Sick => Some(Absence::Sick),
            Self::Vacation => Some(Absence::Vacation),
            Self::Holiday => Some(Absence::Holiday),
            Self::Work | Self::Break => None,
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

impl Day {
    pub fn is_empty(&self) -> bool {
        self.periods.is_empty()
    }

    pub fn has_open_period(&self) -> bool {
        self.periods.iter().any(Period::is_open)
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

    /// Removes the period at `index`, if there is one.
    pub fn remove_period(&mut self, index: usize) {
        if index < self.periods.len() {
            self.periods.remove(index);
        }
    }

    /// Whether there is a gap between the period at `index` and the next one.
    pub fn has_implied_break_after(&self, index: usize) -> bool {
        let (Some(period), Some(next)) = (self.periods.get(index), self.periods.get(index + 1))
        else {
            return false;
        };
        matches!((period.to, next.from), (Some(to), Some(from)) if from > to)
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

    /// Whether a day before today (from the first recorded day on) is empty or has an open period.
    pub fn has_day_warning_on(&self, date: Date, today: Date) -> bool {
        let day = self.day(date);
        self.is_tracked(date) && date < today && (day.is_empty() || day.has_open_period())
    }

    /// The earliest day with at least one period.
    pub fn first_recorded_day(&self) -> Option<Date> {
        self.days
            .iter()
            .find(|(_, day)| !day.is_empty())
            .map(|(&date, _)| date)
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
