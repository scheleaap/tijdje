use tijdje::domain::TimeOfDay;

#[test]
fn four_digits_parse_as_hhmm() {
    assert_eq!(TimeOfDay::parse_hhmm("1235"), TimeOfDay::new(12, 35));
    assert_eq!(TimeOfDay::parse_hhmm("0000"), TimeOfDay::new(0, 0));
    assert_eq!(TimeOfDay::parse_hhmm("2359"), TimeOfDay::new(23, 59));
}

#[test]
fn out_of_range_times_are_rejected() {
    assert_eq!(TimeOfDay::parse_hhmm("2400"), None);
    assert_eq!(TimeOfDay::parse_hhmm("1260"), None);
    assert_eq!(TimeOfDay::parse_hhmm("2575"), None);
}

#[test]
fn anything_but_exactly_four_digits_is_rejected() {
    for input in ["", "935", "09350", "12:35", "12a5", " 935"] {
        assert_eq!(TimeOfDay::parse_hhmm(input), None, "{input:?}");
    }
}

#[test]
fn pasted_times_may_use_a_colon_and_a_one_digit_hour() {
    assert_eq!(TimeOfDay::parse_pasted("12:35"), TimeOfDay::new(12, 35));
    assert_eq!(TimeOfDay::parse_pasted("9:35"), TimeOfDay::new(9, 35));
    assert_eq!(TimeOfDay::parse_pasted("1235"), TimeOfDay::new(12, 35));
}

#[test]
fn other_pasted_text_is_not_a_time() {
    for input in [
        "1:2:3:5",
        ":1235",
        "12:3",
        "123:45",
        "99:99",
        "half past nine",
        "",
    ] {
        assert_eq!(TimeOfDay::parse_pasted(input), None, "{input:?}");
    }
}
