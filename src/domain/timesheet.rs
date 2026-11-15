use std::collections::BTreeMap;

use jiff::civil::Date;

use super::{Minutes, TimeOfDay, calendar};

/// Periods starting before this time have an unusual time.
const EARLIEST_USUAL_START: TimeOfDay = TimeOfDay::new(6, 0).expect("valid time");
/// Periods ending after this time have an unusual time.
const LATEST_USUAL_END: TimeOfDay = TimeOfDay::new(20, 0).expect("valid time");

/// 7h12m.
pub const DEFAULT_REGULAR_EXPECTED_WORKING_TIME: Minutes = Minutes(432);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodType {
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
