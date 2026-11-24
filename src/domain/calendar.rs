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
        dutch_month(date)
    )
}

fn dutch_month(date: Date) -> &'static str {
    MONTHS[usize::try_from(date.month() - 1).expect("month is 1-12")]
}

fn is_weekend(date: Date) -> bool {
    matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
}

fn add_days(date: Date, days: i32) -> Date {
    date.checked_add(days.days())
        .expect("date stays within jiff's supported range")
}

/// Formats a date like `ma 2 nov`, without padding the day.
pub fn format_dutch_short(date: Date) -> String {
    format!(
        "{} {} {}",
        dutch_weekday(date.weekday()),
        date.day(),
        dutch_month(date)
    )
}

/// The day selected on startup: today, or the previous Friday on a weekend.
pub fn start_day(today: Date) -> Date {
    weekday_on_or_before(today)
}

/// `date` itself on a weekday, or the Friday before on a weekend.
pub fn weekday_on_or_before(date: Date) -> Date {
    if is_weekend(date) {
        previous_weekday(date)
    } else {
        date
    }
}

/// The years a typed date may have. The endless day list needs the days
/// around the selected day, so the last of jiff's years is left out; year
/// 0 isn't a four-digit year anyone types.
const TYPED_YEARS: std::ops::RangeInclusive<i16> = 1..=9998;

/// Parses a date typed as `YYYY-MM-DD`, `DD MMM` or `DD MMM YYYY`, with a
/// Dutch month abbreviation (`jan`…`dec`, in any case) and the day as one
/// or two digits. Without a year, the date is in `today`'s year.
pub fn parse_date(input: &str, today: Date) -> Option<Date> {
    let words: Vec<&str> = input.split_whitespace().collect();
    let (year, month, day) = match words.as_slice() {
        [iso] => parse_iso_parts(iso),
        [day, month] => Some((today.year(), parse_month(month)?, parse_digits(day, 1..=2)?)),
        [day, month, year] => Some((
            parse_digits(year, 4..=4)?,
            parse_month(month)?,
            parse_digits(day, 1..=2)?,
        )),
        _ => None,
    }?;
    TYPED_YEARS
        .contains(&year)
        .then(|| Date::new(year, month, day).ok())
        .flatten()
}

/// The year, month and day of `YYYY-MM-DD`.
fn parse_iso_parts(text: &str) -> Option<(i16, i8, i8)> {
    let mut parts = text.split('-');
    let year = parse_digits(parts.next()?, 4..=4)?;
    let month = parse_digits(parts.next()?, 2..=2)?;
    let day = parse_digits(parts.next()?, 2..=2)?;
    parts.next().is_none().then_some((year, month, day))
}

/// The month (1–12) of a Dutch month abbreviation.
fn parse_month(text: &str) -> Option<i8> {
    let index = MONTHS
        .iter()
        .position(|month| month.eq_ignore_ascii_case(text))?;
    i8::try_from(index + 1).ok()
}

/// A number written with only ASCII digits, as many as `len` allows.
fn parse_digits<T: std::str::FromStr>(
    text: &str,
    len: std::ops::RangeInclusive<usize>,
) -> Option<T> {
    let digits_only = text.bytes().all(|b| b.is_ascii_digit());
    (digits_only && len.contains(&text.len()))
        .then(|| text.parse().ok())
        .flatten()
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
