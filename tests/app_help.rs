mod common;

use common::*;
use jiff::civil::date;
use ratatui::{buffer::Buffer, crossterm::event::KeyCode, style::Color};
use tijdje::app::{App, Outcome};

/// A screen height at which the whole Help window fits.
const TALL: u16 = 60;
/// The smallest screen height, at which the Help window doesn't fit.
const SHORT: u16 = 24;
/// The lines the Help window shows at the short height: 24 rows less the
/// function-key bar, a margin row, the shadow row and the frame's two rows.
const SHORT_PAGE: usize = 19;
const SCROLL_HINT: &str = "↑↓ PgUp/PgDn scroll";

/// The app on Friday 13 Nov of the sample with the Help window open.
fn app_with_help() -> App {
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    press(&mut app, KeyCode::F(1));
    app
}

/// The screen rows at a width of 120 and `height` rows.
fn screen(app: &mut App, height: u16) -> Vec<String> {
    rows(&render(app, 120, height))
}

/// The short screen with the Help window open, after pressing `keys`; the
/// screen is drawn before each key, as in the real application.
fn scrolled(keys: &[KeyCode]) -> Vec<String> {
    let mut app = app_with_help();
    for &key in keys {
        screen(&mut app, SHORT);
        press(&mut app, key);
    }
    screen(&mut app, SHORT)
}

/// Where `text` is on the screen, as (column, row).
fn position_of(buffer: &Buffer, text: &str) -> (u16, u16) {
    let y = u16::try_from(row_index(&rows(buffer), text)).unwrap();
    (column_of(buffer, y, text).unwrap(), y)
}

#[test]
fn f1_opens_the_help_window_over_the_tracking_screen() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 13));

    // When
    press(&mut app, KeyCode::F(1));

    // Then
    assert!(app.shows_help());
    let rows = screen(&mut app, TALL);
    assert!(rows[0].contains("Days: 2026"), "{:?}", rows[0]);
    assert!(rows[0].contains("Periods: "), "{:?}", rows[0]);
    row_containing(&rows, "─ Help ─");
}

#[test]
fn esc_f1_and_f2_close_the_help_window() {
    for key in [KeyCode::Esc, KeyCode::F(1), KeyCode::F(2)] {
        // Given
        let mut app = app_with_help();

        // When
        press(&mut app, key);

        // Then
        assert!(!app.shows_help(), "{key:?}");
        assert!(!screen(&mut app, TALL).join("\n").contains("─ Help ─"));
    }
}

#[test]
fn f2_without_the_help_window_does_nothing() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let before = screen(&mut app, SHORT);

    // When
    press(&mut app, KeyCode::F(2));

    // Then
    assert!(!app.shows_help());
    assert_eq!(screen(&mut app, SHORT), before);
}

#[test]
fn the_shortcuts_are_grouped_in_the_spec_order() {
    // Given
    let mut app = app_with_help();

    // When
    let rows = screen(&mut app, TALL);

    // Then
    let groups = [
        "General",
        "Day navigation",
        "Field navigation",
        "Field editing",
        "Period definition",
        "Period management",
    ];
    let indexes: Vec<usize> = groups.iter().map(|g| row_index(&rows, g)).collect();
    assert!(indexes.is_sorted(), "{indexes:?}");
}

#[test]
fn shortcuts_are_listed_in_their_group() {
    // Given: a few representative keys per group
    let mut app = app_with_help();
    let groups: [(&str, &[&str]); 6] = [
        ("General", &["F1 ", "Ctrl+S ", "Esc "]),
        ("Day navigation", &["Ctrl+PgUp/PgDn", "Ctrl+G"]),
        ("Field navigation", &[" PgUp/PgDn ", " Home/End "]),
        ("Field editing", &["Home/End, Ctrl+A/E", "Backspace/Del"]),
        ("Period definition", &["Alt+0/1/2/3"]),
        ("Period management", &["Ctrl+Del"]),
    ];

    // When
    let rows = screen(&mut app, TALL);

    // Then
    for (i, (group, keys)) in groups.iter().enumerate() {
        let start = row_index(&rows, group);
        let end = groups
            .get(i + 1)
            .map_or(rows.len(), |(next, _)| row_index(&rows, next));
        for key in *keys {
            assert!(
                rows[start..end].iter().any(|row| row.contains(key)),
                "{key:?} is not in {group}"
            );
        }
    }
}

#[test]
fn removed_and_merged_shortcuts_are_gone() {
    // Given
    let mut app = app_with_help();

    // When
    let text = screen(&mut app, TALL).join("\n");

    // Then
    for removed in [
        "Alt+Enter",
        "Alt+Del",
        "Alt+p/n",
        "Save/quit",
        "F1; F2",
        "(Ctrl+)",
    ] {
        assert!(!text.contains(removed), "{removed} is gone");
    }
}

#[test]
fn ctrl_s_and_ctrl_q_have_a_line_each() {
    // Given
    let mut app = app_with_help();

    // When
    let rows = screen(&mut app, TALL);

    // Then
    assert!(row_containing(&rows, "Ctrl+S ").contains("Save"));
    assert!(row_containing(&rows, "Ctrl+Q ").contains("Quit"));
    assert_ne!(row_index(&rows, "Ctrl+S "), row_index(&rows, "Ctrl+Q "));
}

#[test]
fn the_project_suggestion_keys_have_a_line_each() {
    // Given
    let mut app = app_with_help();

    // When
    let rows = screen(&mut app, TALL);

    // Then
    assert!(row_containing(&rows, " → ").contains("fill in the project suggestion, keep editing"));
    assert!(
        row_containing(&rows, "take the project suggestion, accept and move on")
            .contains("Tab/Shift-Tab, Ctrl+Enter")
    );
    assert!(
        row_containing(&rows, "While editing: accept and move on")
            .contains("Tab/Shift-Tab, Ctrl+Enter")
    );
}

#[test]
fn the_help_window_has_midnight_commanders_user_menu_colors() {
    // Given
    let mut app = app_with_help();

    // When
    let buffer = render(&mut app, 120, TALL);

    // Then
    let (title_x, title_y) = position_of(&buffer, "Help ─");
    let title = &buffer[(title_x, title_y)];
    assert_eq!((title.fg, title.bg), (Color::Yellow, Color::Cyan));
    let corner_x = (0..title_x)
        .rev()
        .find(|&x| buffer[(x, title_y)].symbol() == "┌")
        .expect("a single-line frame");
    assert_eq!(buffer[(corner_x, title_y)].bg, Color::Cyan);
    let (text_x, text_y) = position_of(&buffer, "Ctrl+Del");
    let text = &buffer[(text_x, text_y)];
    assert_eq!((text.fg, text.bg), (Color::White, Color::Cyan));
    let bottom_y = (title_y..TALL)
        .find(|&y| buffer[(corner_x, y)].symbol() == "└")
        .expect("a bottom frame");
    let shadow = &buffer[(corner_x + 2, bottom_y + 1)];
    assert_eq!(shadow.bg, Color::Black);
}

#[test]
fn the_bottom_frame_shows_how_to_scroll_if_the_list_does_not_fit() {
    // Given
    let mut app = app_with_help();

    // When
    let rows = screen(&mut app, SHORT);

    // Then
    assert!(row_containing(&rows, SCROLL_HINT).contains('└'));
    assert!(!rows.join("\n").contains("Period management"));
}

#[test]
fn the_bottom_frame_shows_no_scroll_hint_if_the_list_fits() {
    // Given
    let mut app = app_with_help();

    // When
    let text = screen(&mut app, TALL).join("\n");

    // Then
    assert!(!text.contains(SCROLL_HINT));
}

#[test]
fn end_scrolls_to_the_bottom() {
    // When
    let text = scrolled(&[KeyCode::End]).join("\n");

    // Then
    assert!(text.contains("Period management"));
    assert!(text.contains("Ctrl+Del"));
    assert!(!text.contains("General"));
}

#[test]
fn home_scrolls_to_the_top() {
    // When
    let rows = scrolled(&[KeyCode::End, KeyCode::Home]);

    // Then
    assert_eq!(rows, scrolled(&[]));
}

#[test]
fn down_scrolls_one_line() {
    // Given
    let top = scrolled(&[]);

    // When
    let rows = scrolled(&[KeyCode::Down]);

    // Then
    assert_eq!(row_index(&rows, "F1 "), row_index(&top, "General"));
}

#[test]
fn up_scrolls_one_line_back() {
    // When
    let rows = scrolled(&[KeyCode::Down, KeyCode::Down, KeyCode::Up]);

    // Then
    assert_eq!(rows, scrolled(&[KeyCode::Down]));
}

#[test]
fn page_down_scrolls_a_page() {
    // When
    let rows = scrolled(&[KeyCode::PageDown]);

    // Then
    assert_eq!(rows, scrolled(&[KeyCode::Down; SHORT_PAGE]));
}

#[test]
fn page_up_scrolls_a_page_back() {
    // When
    let rows = scrolled(&[KeyCode::Down, KeyCode::PageDown, KeyCode::PageUp]);

    // Then
    assert_eq!(rows, scrolled(&[KeyCode::Down]));
}

#[test]
fn down_stops_at_the_bottom() {
    // When
    let rows = scrolled(&[KeyCode::End, KeyCode::Down]);

    // Then
    assert_eq!(rows, scrolled(&[KeyCode::End]));
}

#[test]
fn page_down_stops_at_the_bottom() {
    // When
    let rows = scrolled(&[KeyCode::End, KeyCode::PageDown, KeyCode::Up]);

    // Then: one line up from the bottom, not from beyond it
    assert_eq!(rows, scrolled(&[KeyCode::End, KeyCode::Up]));
    assert_ne!(rows, scrolled(&[KeyCode::End]));
}

#[test]
fn keys_of_the_tracking_screen_do_nothing_while_the_help_window_is_open() {
    // Given
    let mut app = app_with_help();
    let focus = app.focus();

    // When
    type_text(&mut app, "09");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::Tab);
    ctrl_key(&mut app, KeyCode::Down);
    alt_key(&mut app, KeyCode::Char('b'));
    ctrl(&mut app, 'g');

    // Then
    assert!(app.shows_help());
    assert!(!app.is_editing());
    assert_eq!(app.focus(), focus);
    assert_eq!(app.selected_day(), date(2026, 11, 13));
    assert!(!app.has_unsaved_changes());
}

#[test]
fn ctrl_s_does_nothing_while_the_help_window_is_open() {
    // Given: Mon 2 Nov with its first From changed
    let (mut app, fs) = sample_on(date(2026, 11, 2));
    type_text(&mut app, "0815");
    press(&mut app, KeyCode::Enter);
    press(&mut app, KeyCode::F(1));

    // When
    ctrl(&mut app, 's');

    // Then
    assert!(app.has_unsaved_changes());
    assert_eq!(fs.get(TIMESHEET).as_deref(), Some(SAMPLE));
    assert!(!screen(&mut app, SHORT).join("\n").contains("Saved"));
}

#[test]
fn ctrl_q_does_nothing_while_the_help_window_is_open() {
    // Given
    let mut app = app_with_help();

    // When
    let outcome = ctrl(&mut app, 'q');

    // Then
    assert_eq!(outcome, Outcome::Continue);
    assert!(app.shows_help());
}

#[test]
fn the_function_key_bar_is_on_the_bottom_line() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 13));

    // When
    let rows = screen(&mut app, SHORT);

    // Then
    assert!(rows[23].starts_with(" 1Help"));
    assert!(rows[23].contains(" 2Track"));
    assert!(rows[23].contains("10"));
}

#[test]
fn the_function_key_slots_stretch_evenly_over_the_full_width() {
    // Given
    let mut app = app_with(SAMPLE, date(2026, 11, 13));
    let width = 125;

    // When
    let buffer = render(&mut app, width, SHORT);

    // Then
    let row = &rows(&buffer)[23];
    let starts: Vec<u16> = (1..=10)
        .map(|n| column_of(&buffer, 23, &format!("{n:>2}")).unwrap_or_else(|| panic!("{row:?}")))
        .chain([width])
        .collect();
    let widths: Vec<u16> = starts.windows(2).map(|w| w[1] - w[0]).collect();
    assert!(widths.iter().all(|&w| w == 12 || w == 13), "{widths:?}");
    assert_eq!(buffer[(width - 1, 23)].bg, Color::Cyan);
}
