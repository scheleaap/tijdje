use tijdje::domain::Minutes;

#[test]
fn durations_are_formatted_as_hours_and_two_digit_minutes() {
    assert_eq!(Minutes(115).format_duration(), "1h55m");
    assert_eq!(Minutes(432).format_duration(), "7h12m");
    assert_eq!(Minutes(5).format_duration(), "0h05m");
    assert_eq!(Minutes(600).format_duration(), "10h00m");
}

#[test]
fn balances_are_formatted_with_a_sign() {
    assert_eq!(Minutes(45).format_balance(), "+0h45m");
    assert_eq!(Minutes(-72).format_balance(), "-1h12m");
    assert_eq!(Minutes(-633).format_balance(), "-10h33m");
}

#[test]
fn a_zero_balance_has_no_sign() {
    assert_eq!(Minutes(0).format_balance(), "0h00m");
}

#[test]
fn negative_durations_keep_minutes_positive() {
    assert_eq!(Minutes(-75).format_duration(), "-1h15m");
}
