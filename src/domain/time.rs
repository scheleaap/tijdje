use std::{
    iter::Sum,
    ops::{Add, Sub},
};

/// A time of day, stored as minutes since midnight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TimeOfDay(u16);

impl TimeOfDay {
    pub const fn new(hour: u16, minute: u16) -> Option<Self> {
        if hour < 24 && minute < 60 {
            Some(Self(hour * 60 + minute))
        } else {
            None
        }
    }

    pub fn minutes_since_midnight(self) -> u16 {
        self.0
    }

    /// Parses a time typed as exactly four digits HHMM, e.g. `1235`.
    pub fn parse_hhmm(input: &str) -> Option<Self> {
        if input.len() != 4 || !input.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        Self::new(input[..2].parse().ok()?, input[2..].parse().ok()?)
    }

    /// Formats the time like `1235`, as typed in the From and To fields.
    pub fn format_hhmm(self) -> String {
        format!("{:02}{:02}", self.0 / 60, self.0 % 60)
    }
}

/// A signed amount of time in minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Minutes(pub i32);

impl Minutes {
    /// Formats a duration like `1h55m`.
    pub fn format_duration(self) -> String {
        let sign = if self.0 < 0 { "-" } else { "" };
        let abs = self.0.unsigned_abs();
        format!("{sign}{}h{:02}m", abs / 60, abs % 60)
    }

    /// Formats a balance like `+0h45m` or `-1h12m`; zero has no sign.
    pub fn format_balance(self) -> String {
        let sign = if self.0 > 0 { "+" } else { "" };
        format!("{sign}{}", self.format_duration())
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

impl Sub for Minutes {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}

impl Sum for Minutes {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::default(), Add::add)
    }
}
