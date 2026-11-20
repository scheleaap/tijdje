//! The clock that says which day is today, injected so tests can use a fake one.

use jiff::civil::Date;

pub trait Clock {
    /// Today's date.
    fn today(&self) -> Date;
}

/// The system clock, in the system time zone.
pub struct SystemClock;

impl Clock for SystemClock {
    fn today(&self) -> Date {
        jiff::Zoned::now().date()
    }
}
