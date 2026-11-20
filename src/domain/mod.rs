//! Timesheet domain: pure rules, no I/O.

pub mod calendar;
mod time;
mod timesheet;

pub use time::{Minutes, TimeOfDay};
pub use timesheet::{
    Absence, ConsistencyProblem, Day, InconsistentDay, Period, PeriodIndicator, PeriodType,
    Timesheet,
};
