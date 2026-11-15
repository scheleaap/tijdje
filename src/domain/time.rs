use std::{
    iter::Sum,
    ops::{Add, Sub},
};

/// A time of day, stored as minutes since midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeOfDay(u16);

impl TimeOfDay {
    pub fn new(hour: u16, minute: u16) -> Option<Self> {
        (hour < 24 && minute < 60).then_some(Self(hour * 60 + minute))
    }

    pub fn minutes_since_midnight(self) -> u16 {
        self.0
    }
}

/// A signed amount of time in minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Minutes(pub i32);

impl Minutes {
    /// Formats an unsigned duration like `1h55m`.
    pub fn format_duration(self) -> String {
        format!("{}h{:02}m", self.0 / 60, self.0 % 60)
    }
}

impl Sub for TimeOfDay {
    type Output = Minutes;

    fn sub(self, earlier: Self) -> Minutes {
        Minutes(i32::from(self.0) - i32::from(earlier.0))
    }
}

impl Add for Minutes {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

impl Sum for Minutes {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}
