//! Timesheet domain: pure rules, no I/O.

pub mod calendar;
mod time;
mod timesheet;

pub use time::{Minutes, TimeOfDay};
pub use timesheet::{
    Absence, ConsistencyProblem, Day, Extent, ExtentKind, InconsistentDay, PartOfDay, Period,
    PeriodIndicator, PeriodType, Timesheet,
};
