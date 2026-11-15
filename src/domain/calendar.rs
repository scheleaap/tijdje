//! Weekday navigation and Dutch date formatting.

use jiff::{
    ToSpan,
    civil::{Date, Weekday},
};

const MONTHS: [&str; 12] = [
    "jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
];

fn dutch_weekday(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Monday => "ma",
        Weekday::Tuesday => "di",
        Weekday::Wednesday => "wo",
        Weekday::Thursday => "do",
        Weekday::Friday => "vr",
        Weekday::Saturday => "za",
        Weekday::Sunday => "zo",
    }
}

/// Formats a date like `ma 12 nov`, padding the day to two characters.
pub fn format_dutch(date: Date) -> String {
    format!(
        "{} {:>2} {}",
        dutch_weekday(date.weekday()),
        date.day(),
        MONTHS[usize::try_from(date.month() - 1).expect("month is 1-12")]
    )
}

fn is_weekend(date: Date) -> bool {
    matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
}

fn add_days(date: Date, days: i32) -> Date {
    date.checked_add(days.days())
        .expect("date stays within jiff's supported range")
}

/// The day selected on startup: today, or the previous Friday on a weekend.
pub fn start_day(today: Date) -> Date {
    if is_weekend(today) {
        previous_weekday(today)
    } else {
        today
    }
}

/// Steps one day at a time in `direction` (+1 or -1) until a weekday is reached.
fn step_to_weekday(date: Date, direction: i32) -> Date {
    let mut day = add_days(date, direction);
    while is_weekend(day) {
        day = add_days(day, direction);
    }
    day
}

pub fn next_weekday(date: Date) -> Date {
    step_to_weekday(date, 1)
}

pub fn previous_weekday(date: Date) -> Date {
    step_to_weekday(date, -1)
}

pub fn next_week(date: Date) -> Date {
    add_days(date, 7)
}

pub fn previous_week(date: Date) -> Date {
    add_days(date, -7)
}

/// The line drawn between a listed day and the next listed day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Separator {
    None,
    Week,
    Year,
}

/// The separator between `date` and the next weekday.
pub fn separator_after(date: Date) -> Separator {
    let next = next_weekday(date);
    if next.year() != date.year() {
        Separator::Year
    } else if date.weekday() == Weekday::Friday {
        Separator::Week
    } else {
        Separator::None
    }
}
