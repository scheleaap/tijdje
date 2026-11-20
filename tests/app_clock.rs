mod common;

use common::*;
use jiff::civil::date;
use tijdje::app::App;

fn screen(app: &mut App) -> Vec<String> {
    rows(&render(app, 120, 40))
}

#[test]
fn after_midnight_balances_use_the_new_today_and_the_selected_day_stays() {
    let clock = FakeClock::new(date(2026, 11, 12));
    let (mut app, _fs) = app_with_clock(SAMPLE, &clock);
    assert!(!row_containing(&screen(&mut app), "vr 13 nov").contains('h'));

    clock.set(date(2026, 11, 13));
    app.refresh_today();

    let row = row_containing(&screen(&mut app), "vr 13 nov").to_owned();
    assert!(row.contains("-7h12m") && row.contains("-10h33m"), "{row:?}");
    assert_eq!(app.selected_day(), date(2026, 11, 12));
}

#[test]
fn after_midnight_day_warnings_use_the_new_today() {
    let clock = FakeClock::new(date(2026, 11, 11));
    let (mut app, _fs) = app_with_clock(SAMPLE, &clock);
    assert!(!row_containing(&screen(&mut app), "wo 11 nov").contains("⚠️"));

    clock.set(date(2026, 11, 12));
    app.refresh_today();

    assert!(row_containing(&screen(&mut app), "wo 11 nov").contains("⚠️"));
}

#[test]
fn after_midnight_an_open_period_from_yesterday_fails_the_check() {
    let clock = FakeClock::new(SAMPLE_TODAY);
    let (mut app, _fs) = app_with_clock(SAMPLE, &clock);
    assert!(!row_containing(&screen(&mut app), "di 10 nov").contains("⚠️"));

    clock.set(date(2026, 11, 11));
    app.refresh_today();

    assert!(row_containing(&screen(&mut app), "di 10 nov").contains("⚠️"));
    ctrl(&mut app, 's');
    let text = screen(&mut app).join("\n");
    assert!(
        text.contains("di 10 nov: Open period 1230– on a past day"),
        "{text}"
    );
}

#[test]
fn a_key_press_uses_the_current_today_even_without_a_redraw() {
    let clock = FakeClock::new(SAMPLE_TODAY);
    let (mut app, _fs) = app_with_clock(SAMPLE, &clock);
    screen(&mut app);

    // Midnight passes while the app waits for a key; no refresh in between.
    clock.set(date(2026, 11, 11));
    ctrl(&mut app, 's');

    let text = screen(&mut app).join("\n");
    assert!(
        text.contains("di 10 nov: Open period 1230– on a past day"),
        "{text}"
    );
}
