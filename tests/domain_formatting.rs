use tijdje::domain::Minutes;

#[test]
fn durations_are_formatted_as_hours_and_two_digit_minutes() {
    assert_eq!(Minutes(115).format_duration(), "1h55m");
    assert_eq!(Minutes(432).format_duration(), "7h12m");
    assert_eq!(Minutes(5).format_duration(), "0h05m");
    assert_eq!(Minutes(600).format_duration(), "10h00m");
}
