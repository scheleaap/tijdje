use tijdje::domain::{Day, Minutes, Period, PeriodType, TimeOfDay};

fn time(hhmm: u16) -> Option<TimeOfDay> {
    TimeOfDay::new(hhmm / 100, hhmm % 100)
}

fn period(from: u16, to: Option<u16>, period_type: PeriodType) -> Period {
    Period {
        from: time(from),
        to: to.and_then(time),
        project: None,
        notes: None,
        period_type,
    }
}

#[test]
fn worked_time_sums_work_periods() {
    let day = Day {
        periods: vec![
            period(900, Some(1200), PeriodType::Work),
            period(1300, Some(1612), PeriodType::Work),
        ],
        ..Day::default()
    };
    assert_eq!(day.worked_time(), Minutes(372)); // 3h00m + 3h12m
}

#[test]
fn worked_time_excludes_non_working_periods() {
    let day = Day {
        periods: vec![
            period(800, Some(1200), PeriodType::Work),
            period(1200, Some(1230), PeriodType::Break),
            period(1230, Some(1400), PeriodType::Sick),
            period(1400, Some(1500), PeriodType::Vacation),
            period(1500, Some(1600), PeriodType::Holiday),
        ],
        ..Day::default()
    };
    assert_eq!(day.worked_time(), Minutes(240));
}

#[test]
fn worked_time_excludes_open_periods() {
    let day = Day {
        periods: vec![
            period(800, Some(1000), PeriodType::Work),
            period(1000, None, PeriodType::Work),
        ],
        ..Day::default()
    };
    assert_eq!(day.worked_time(), Minutes(120));
}
