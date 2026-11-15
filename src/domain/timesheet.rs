use std::collections::BTreeMap;

use jiff::civil::Date;

use super::{Minutes, TimeOfDay};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeriodType {
    Work,
    Break,
    Vacation,
    Sick,
    Holiday,
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

impl Day {
    pub fn worked_time(&self) -> Minutes {
        self.periods
            .iter()
            .filter(|p| p.period_type == PeriodType::Work)
            .filter_map(Period::duration)
            .sum()
    }
}

impl Period {
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
    /// The worked time on `date`, or `None` when nothing is recorded that day.
    pub fn worked_time_on(&self, date: Date) -> Option<Minutes> {
        self.days
            .get(&date)
            .filter(|day| !day.periods.is_empty())
            .map(Day::worked_time)
    }
}
