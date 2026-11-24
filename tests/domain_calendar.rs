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

#[test]
fn a_weekday_is_its_own_weekday_on_or_before() {
    assert_eq!(
        calendar::weekday_on_or_before(date(2026, 11, 12)),
        date(2026, 11, 12)
    );
}

#[test]
fn a_weekend_day_goes_back_to_the_friday_before() {
    assert_eq!(
        calendar::weekday_on_or_before(date(2026, 11, 14)),
        date(2026, 11, 13)
    );
    assert_eq!(
        calendar::weekday_on_or_before(date(2026, 11, 15)),
        date(2026, 11, 13)
    );
}

const TODAY: jiff::civil::Date = date(2026, 11, 24);

#[test]
fn an_iso_date_is_parsed() {
    assert_eq!(
        calendar::parse_date("2025-03-07", TODAY),
        Some(date(2025, 3, 7))
    );
}

#[test]
fn a_day_and_dutch_month_is_in_the_current_year() {
    assert_eq!(
        calendar::parse_date("15 nov", TODAY),
        Some(date(2026, 11, 15))
    );
    assert_eq!(calendar::parse_date("3 mrt", TODAY), Some(date(2026, 3, 3)));
}

#[test]
fn a_day_dutch_month_and_year_is_parsed() {
    assert_eq!(
        calendar::parse_date("15 nov 2025", TODAY),
        Some(date(2025, 11, 15))
    );
}

#[test]
fn every_dutch_month_abbreviation_is_known() {
    let months = [
        "jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec",
    ];
    for (month, abbreviation) in (1..=12).zip(months) {
        assert_eq!(
            calendar::parse_date(&format!("01 {abbreviation} 2026"), TODAY),
            Some(date(2026, month, 1)),
            "{abbreviation}"
        );
    }
}

#[test]
fn case_and_surrounding_spaces_do_not_matter() {
    assert_eq!(
        calendar::parse_date("  15  Nov 2025 ", TODAY),
        Some(date(2025, 11, 15))
    );
    assert_eq!(
        calendar::parse_date(" 2025-11-15 ", TODAY),
        Some(date(2025, 11, 15))
    );
}

#[test]
fn english_month_abbreviations_are_rejected() {
    for input in ["1 mar", "1 may", "1 oct"] {
        assert_eq!(calendar::parse_date(input, TODAY), None, "{input:?}");
    }
}

#[test]
fn dates_that_do_not_exist_are_rejected() {
    for input in [
        "2026-02-29",
        "2026-13-01",
        "2026-00-10",
        "31 apr",
        "0 jan",
        "32 dec",
    ] {
        assert_eq!(calendar::parse_date(input, TODAY), None, "{input:?}");
    }
}

#[test]
fn other_text_is_not_a_date() {
    for input in [
        "",
        "today",
        "15",
        "nov",
        "nov 15",
        "15 nov.",
        "2026-1-5",
        "15 november",
        "15 nov 25",
        "15 nov 2025 extra",
        "115 nov",
        "2025-3-7",
        "25-03-07",
        "2025/03/07",
        "+15 nov",
        "15 nov +202",
        "2025-+3-07",
        "15nov",
    ] {
        assert_eq!(calendar::parse_date(input, TODAY), None, "{input:?}");
    }
}

#[test]
fn years_leave_room_for_the_endless_day_list() {
    assert_eq!(
        calendar::parse_date("31 dec 9998", TODAY),
        Some(date(9998, 12, 31))
    );
    assert_eq!(
        calendar::parse_date("0001-01-01", TODAY),
        Some(date(1, 1, 1))
    );
    assert_eq!(calendar::parse_date("9999-01-04", TODAY), None);
    assert_eq!(calendar::parse_date("0000-01-04", TODAY), None);
}
