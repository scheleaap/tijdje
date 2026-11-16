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
