use jiff::civil::date;
use tijdje::domain::calendar::{self, Separator};

#[test]
fn dates_are_formatted_in_dutch() {
    assert_eq!(calendar::format_dutch(date(2026, 11, 12)), "do 12 nov");
    assert_eq!(calendar::format_dutch(date(2026, 3, 16)), "ma 16 mrt");
    assert_eq!(calendar::format_dutch(date(2026, 5, 29)), "vr 29 mei");
    assert_eq!(calendar::format_dutch(date(2026, 10, 27)), "di 27 okt");
}

#[test]
fn single_digit_days_are_padded_so_columns_align() {
    assert_eq!(calendar::format_dutch(date(2026, 11, 4)), "wo  4 nov");
}

#[test]
fn start_day_is_today_on_a_weekday() {
    assert_eq!(calendar::start_day(date(2026, 11, 12)), date(2026, 11, 12));
}

#[test]
fn start_day_is_previous_friday_on_a_weekend() {
    assert_eq!(calendar::start_day(date(2026, 11, 14)), date(2026, 11, 13));
    assert_eq!(calendar::start_day(date(2026, 11, 15)), date(2026, 11, 13));
}

#[test]
fn next_and_previous_weekday_skip_weekends() {
    assert_eq!(
        calendar::next_weekday(date(2026, 11, 13)),
        date(2026, 11, 16)
    );
    assert_eq!(
        calendar::next_weekday(date(2026, 11, 10)),
        date(2026, 11, 11)
    );
    assert_eq!(
        calendar::previous_weekday(date(2026, 11, 16)),
        date(2026, 11, 13)
    );
    assert_eq!(
        calendar::previous_weekday(date(2026, 11, 11)),
        date(2026, 11, 10)
    );
}

#[test]
fn week_steps_keep_the_weekday() {
    assert_eq!(calendar::next_week(date(2026, 11, 11)), date(2026, 11, 18));
    assert_eq!(
        calendar::previous_week(date(2026, 11, 11)),
        date(2026, 11, 4)
    );
}

#[test]
fn no_separator_within_a_week() {
    assert_eq!(
        calendar::separator_after(date(2026, 11, 11)),
        Separator::None
    );
}

#[test]
fn single_line_between_friday_and_monday() {
    assert_eq!(
        calendar::separator_after(date(2026, 11, 13)),
        Separator::Week
    );
}

#[test]
fn double_line_between_years_within_a_week() {
    // Thursday 31 Dec 2026 → Friday 1 Jan 2027
    assert_eq!(
        calendar::separator_after(date(2026, 12, 31)),
        Separator::Year
    );
}

#[test]
fn double_line_wins_when_the_year_changes_over_a_weekend() {
    // Friday 31 Dec 2027 → Monday 3 Jan 2028
    assert_eq!(
        calendar::separator_after(date(2027, 12, 31)),
        Separator::Year
    );
}

#[test]
fn short_dates_are_not_padded() {
    assert_eq!(calendar::format_dutch_short(date(2026, 11, 2)), "ma 2 nov");
    assert_eq!(
        calendar::format_dutch_short(date(2026, 11, 12)),
        "do 12 nov"
    );
}
