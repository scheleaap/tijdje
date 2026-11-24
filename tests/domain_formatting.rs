use tijdje::domain::{Minutes, TimeOfDay};

#[test]
fn durations_are_formatted_as_hours_and_two_digit_minutes() {
    assert_eq!(Minutes(115).format_duration(), "1h55m");
    assert_eq!(Minutes(432).format_duration(), "7h12m");
    assert_eq!(Minutes(65).format_duration(), "1h05m");
    assert_eq!(Minutes(600).format_duration(), "10h00m");
}

#[test]
fn durations_under_an_hour_have_no_hours() {
    assert_eq!(Minutes(42).format_duration(), "42m");
    assert_eq!(Minutes(5).format_duration(), "5m");
}

#[test]
fn a_zero_duration_is_zero_minutes() {
    assert_eq!(Minutes(0).format_duration(), "0m");
}

#[test]
fn balances_are_formatted_with_a_sign() {
    assert_eq!(Minutes(45).format_balance(), "+45m");
    assert_eq!(Minutes(-12).format_balance(), "-12m");
    assert_eq!(Minutes(-72).format_balance(), "-1h12m");
    assert_eq!(Minutes(-633).format_balance(), "-10h33m");
}

#[test]
fn a_zero_balance_has_no_sign() {
    assert_eq!(Minutes(0).format_balance(), "0m");
}

#[test]
fn negative_durations_keep_minutes_positive() {
    assert_eq!(Minutes(-75).format_duration(), "-1h15m");
}

#[test]
fn times_are_displayed_as_four_digits_without_colon() {
    assert_eq!(TimeOfDay::new(12, 35).unwrap().format_hhmm(), "1235");
    assert_eq!(TimeOfDay::new(9, 5).unwrap().format_hhmm(), "0905");
    assert_eq!(TimeOfDay::new(0, 0).unwrap().format_hhmm(), "0000");
}

#[test]
fn times_can_be_displayed_with_a_colon() {
    assert_eq!(TimeOfDay::new(9, 5).unwrap().format_with_colon(), "09:05");
    assert_eq!(TimeOfDay::new(18, 0).unwrap().format_with_colon(), "18:00");
}

#[test]
fn adding_minutes_to_a_time_stays_within_the_day() {
    let time = |hour, minute| TimeOfDay::new(hour, minute);
    let noon = time(12, 0).unwrap();
    assert_eq!(noon.checked_add(Minutes(90)), time(13, 30));
    assert_eq!(noon.checked_add(Minutes(-720)), time(0, 0));
    assert_eq!(noon.checked_add(Minutes(-721)), None, "before 00:00");
    assert_eq!(noon.checked_add(Minutes(719)), time(23, 59));
    assert_eq!(noon.checked_add(Minutes(720)), None, "24:00");
}
