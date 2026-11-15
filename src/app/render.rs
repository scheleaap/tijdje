//! Drawing the application state with Norton Commander styling.

use jiff::civil::Date;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use super::{App, Field, Panel, Screen};
use crate::domain::{
    Day, Minutes, Period, PeriodType, TimeOfDay,
    calendar::{self, Separator},
};

const MIN_WIDTH: u16 = 120;
const MIN_HEIGHT: u16 = 24;
const LEFT_PANEL_WIDTH: u16 = 46;

const WORKED_WIDTH: usize = 6;
const DAY_BALANCE_WIDTH: usize = 7;
const TOTAL_BALANCE_WIDTH: usize = 8;

const TIME_WIDTH: usize = 4;
const DURATION_WIDTH: usize = 8;
const PROJECT_WIDTH: usize = 10;
const CHECKBOX_WIDTH: usize = 3;
/// `→ ☕ ⚠️`: implied break (4 cells), a space, unusual time (2 cells).
const INDICATORS_WIDTH: usize = 7;
const GAP: usize = 1;
/// Everything in a period row except the Notes field.
const PERIOD_ROW_FIXED_WIDTH: usize = GAP
    + TIME_WIDTH
    + GAP
    + TIME_WIDTH
    + GAP
    + DURATION_WIDTH
    + GAP
    + PROJECT_WIDTH
    + GAP
    + CHECKBOXES.len() * (GAP + CHECKBOX_WIDTH)
    + GAP
    + INDICATORS_WIDTH;

/// The period-type checkboxes, in Tab order.
const CHECKBOXES: [(Field, PeriodType); 4] = [
    (Field::Break, PeriodType::Break),
    (Field::Vacation, PeriodType::Vacation),
    (Field::Sick, PeriodType::Sick),
    (Field::Holiday, PeriodType::Holiday),
];

/// The emoji for a non-working period type: checkbox headers and absence indicators.
fn period_type_emoji(period_type: PeriodType) -> &'static str {
    match period_type {
        PeriodType::Break => "☕",
        PeriodType::Vacation => "🏖️",
        PeriodType::Sick => "🤒",
        PeriodType::Holiday => "🎄",
        PeriodType::Work => "  ",
    }
}

const PANEL: Style = Style::new().fg(Color::Cyan).bg(Color::Blue);
const SELECTED: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
const INACTIVE_SELECTED: Style = Style::new().fg(Color::Black).bg(Color::Gray);
const INPUT: Style = Style::new().fg(Color::White).bg(Color::Indexed(17));
const KEY_NUMBER: Style = Style::new().fg(Color::White).bg(Color::Black);
const KEY_LABEL: Style = Style::new().fg(Color::Black).bg(Color::Cyan);

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            render_too_small(frame, area);
            return;
        }
        let [main, keys] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        match self.screen {
            Screen::Tracking => self.render_tracking(frame, main),
            Screen::Help => render_help(frame, main),
        }
        render_function_keys(frame, keys);
    }

    fn render_tracking(&mut self, frame: &mut Frame, area: Rect) {
        let [left, right] =
            Layout::horizontal([Constraint::Length(LEFT_PANEL_WIDTH), Constraint::Fill(1)])
                .areas(area);
        self.render_days(frame, left);
        self.render_periods(frame, right);
    }

    fn render_days(&mut self, frame: &mut Frame, area: Rect) {
        let block = panel_block(" Days ", self.panel == Panel::Days);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [header, list] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        let header_text = format!(
            " {:<9}  {:>WORKED_WIDTH$} {:>DAY_BALANCE_WIDTH$} {:>TOTAL_BALANCE_WIDTH$}",
            "Date", "Worked", "Day", "Total"
        );
        frame.render_widget(Paragraph::new(header_text).style(PANEL), header);

        let height = usize::from(list.height);
        let first = self.scroll_to_selected_day(height);
        let width = usize::from(list.width);
        let selected_style = if self.panel == Panel::Days {
            SELECTED
        } else {
            INACTIVE_SELECTED
        };
        let lines: Vec<Line> = list_rows(first)
            .take(height)
            .map(|row| match row {
                ListRow::Day(date) => {
                    let style = if date == self.selected {
                        selected_style
                    } else {
                        PANEL
                    };
                    Line::from(Span::styled(fit(&self.day_row_text(date), width), style))
                }
                ListRow::Separator(Separator::Year) => Line::raw("═".repeat(width)),
                ListRow::Separator(_) => Line::raw("─".repeat(width)),
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).style(PANEL), list);
    }

    fn day_row_text(&self, date: Date) -> String {
        let timesheet = &self.timesheet;
        let worked = timesheet
            .worked_time_on(date)
            .map(Minutes::format_duration)
            .unwrap_or_default();
        let day_balance = timesheet
            .day_balance_on(date, self.today)
            .map(Minutes::format_balance)
            .unwrap_or_default();
        let total_balance = timesheet
            .total_balance_on(date, self.today)
            .map(Minutes::format_balance)
            .unwrap_or_default();
        let absence = timesheet
            .absence_on(date)
            .map_or("  ", |absence| period_type_emoji(absence.into()));
        let warning = if timesheet.has_day_warning_on(date, self.today) {
            "⚠️"
        } else {
            "  "
        };
        format!(
            " {}  {worked:>WORKED_WIDTH$} {day_balance:>DAY_BALANCE_WIDTH$} {total_balance:>TOTAL_BALANCE_WIDTH$}  {absence} {warning}",
            calendar::format_dutch(date)
        )
    }

    /// Adjusts the first visible day so the selected day is visible, and returns it.
    fn scroll_to_selected_day(&mut self, height: usize) -> Date {
        let mut first = self.first_visible_day.unwrap_or_else(|| {
            let mut first = self.selected;
            for _ in 0..height / 3 {
                first = calendar::previous_weekday(first);
            }
            first
        });
        if first > self.selected {
            first = self.selected;
        }
        while line_of(first, self.selected) >= height {
            first = calendar::next_weekday(first);
        }
        self.first_visible_day = Some(first);
        first
    }

    fn render_periods(&mut self, frame: &mut Frame, area: Rect) {
        let title = format!(" Periods: {} ", calendar::format_dutch(self.selected));
        let block = panel_block(&title, self.panel == Panel::Periods);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [header, list, actual_expected] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        let notes_width = usize::from(inner.width).saturating_sub(PERIOD_ROW_FIXED_WIDTH);

        let checkbox_headers: String = CHECKBOXES
            .iter()
            .map(|&(_, period_type)| format!(" {} ", period_type_emoji(period_type)))
            .collect();
        let header_text = format!(
            " {:<TIME_WIDTH$} {:<TIME_WIDTH$} {:<DURATION_WIDTH$} {:<PROJECT_WIDTH$} {:<notes_width$}{checkbox_headers}",
            "From", "To", "Duration", "Project", "Notes"
        );
        frame.render_widget(Paragraph::new(header_text).style(PANEL), header);

        let height = usize::from(list.height);
        let first = self.scroll_to_focused_period(height);
        let day = self.timesheet.day(self.selected);
        let lines: Vec<Line> = day
            .periods
            .iter()
            .enumerate()
            .skip(first)
            .take(height)
            .map(|(index, period)| self.period_line(day, index, period, notes_width))
            .collect();
        frame.render_widget(Paragraph::new(lines).style(PANEL), list);

        frame.render_widget(
            Paragraph::new(format!(" {}", actual_expected_text(day))).style(PANEL),
            actual_expected,
        );
    }

    fn period_line<'a>(
        &self,
        day: &Day,
        index: usize,
        period: &'a Period,
        notes_width: usize,
    ) -> Line<'a> {
        let field = |field: Field, text: &str, width: usize| {
            let style = if self.is_focused(index, field) {
                SELECTED
            } else {
                INPUT
            };
            Span::styled(fit(text, width), style)
        };
        let time = |t: Option<TimeOfDay>| t.map(|t| t.format_hhmm()).unwrap_or_default();
        let duration = period
            .duration()
            .map(Minutes::format_duration)
            .unwrap_or_default();
        let implied_break = if day.has_implied_break_after(index) {
            "→ ☕"
        } else {
            "    "
        };
        let unusual = if period.has_unusual_time() {
            "⚠️"
        } else {
            "  "
        };
        let gap = || Span::raw(" ");
        let mut spans = vec![
            gap(),
            field(Field::From, &time(period.from), TIME_WIDTH),
            gap(),
            field(Field::To, &time(period.to), TIME_WIDTH),
            gap(),
            Span::raw(format!("{duration:<DURATION_WIDTH$}")),
            gap(),
            field(
                Field::Project,
                period.project.as_deref().unwrap_or_default(),
                PROJECT_WIDTH,
            ),
            gap(),
            field(
                Field::Notes,
                period.notes.as_deref().unwrap_or_default(),
                notes_width,
            ),
        ];
        for (field, period_type) in CHECKBOXES {
            let mark = if period.period_type == period_type {
                "[x]"
            } else {
                "[ ]"
            };
            let style = if self.is_focused(index, field) {
                SELECTED
            } else {
                PANEL
            };
            spans.extend([gap(), Span::styled(mark, style)]);
        }
        spans.extend([gap(), Span::raw(format!("{implied_break} {unusual}"))]);
        Line::from(spans)
    }

    fn is_focused(&self, period: usize, field: Field) -> bool {
        self.focus
            .is_some_and(|f| f.period == period && f.field == field)
    }

    /// Adjusts the first visible period so the focused period is visible, and returns it.
    fn scroll_to_focused_period(&mut self, height: usize) -> usize {
        if let Some(focus) = self.focus {
            if focus.period < self.first_visible_period {
                self.first_visible_period = focus.period;
            } else if height > 0 && focus.period >= self.first_visible_period + height {
                self.first_visible_period = focus.period + 1 - height;
            }
        }
        self.first_visible_period
    }
}

fn actual_expected_text(day: &Day) -> String {
    let actual = day.actual_expected_working_time().format_duration();
    let absence = day.absence_time();
    if absence == Minutes(0) {
        format!("Actual expected: {actual}")
    } else {
        format!(
            "Actual expected: {actual} (regular {} − {} absence)",
            day.regular_expected_working_time().format_duration(),
            absence.format_duration()
        )
    }
}

/// Pads or cuts `text` to exactly `width` cells, ending a cut text with `…`.
fn fit(text: &str, width: usize) -> String {
    let text_width = Span::raw(text).width();
    if text_width <= width {
        return format!("{text}{}", " ".repeat(width - text_width));
    }
    let mut fitted = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = Span::raw(c.to_string()).width();
        if used + w + 1 > width {
            break;
        }
        fitted.push(c);
        used += w;
    }
    fitted.push('…');
    format!("{fitted}{}", " ".repeat(width.saturating_sub(used + 1)))
}

/// A line in the left panel's list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListRow {
    Day(Date),
    Separator(Separator),
}

/// The endless list of rows starting at day `first`.
fn list_rows(first: Date) -> impl Iterator<Item = ListRow> {
    std::iter::successors(Some(first), |&date| Some(calendar::next_weekday(date))).flat_map(
        |date| {
            let separator = calendar::separator_after(date);
            std::iter::once(ListRow::Day(date))
                .chain((separator != Separator::None).then_some(ListRow::Separator(separator)))
        },
    )
}

/// The line on which `date` appears when the list starts at `first` (`first <= date`).
fn line_of(first: Date, date: Date) -> usize {
    list_rows(first)
        .position(|row| row == ListRow::Day(date))
        .expect("the list is endless and first <= date")
}

/// A panel frame; the active panel's title is highlighted.
fn panel_block(title: &str, active: bool) -> Block<'_> {
    let title_style = if active { SELECTED } else { PANEL };
    Block::bordered()
        .border_type(BorderType::Double)
        .title(Span::styled(title, title_style))
        .style(PANEL)
}

fn render_function_keys(frame: &mut Frame, area: Rect) {
    let labels = ["Help", "Track", "", "", "", "", "", "", "", ""];
    let key_width = usize::from(area.width) / labels.len();
    let spans: Vec<Span> = labels
        .iter()
        .enumerate()
        .flat_map(|(i, label)| {
            let number = format!("{:>2}", i + 1);
            let label_width = key_width.saturating_sub(number.len());
            [
                Span::styled(number, KEY_NUMBER),
                Span::styled(format!("{label:<label_width$}"), KEY_LABEL),
            ]
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(spans)).style(KEY_NUMBER), area);
}

fn render_help(frame: &mut Frame, area: Rect) {
    let shortcuts = [
        ("Alt+←/→", "Focus the left/right panel"),
        ("↑/↓", "Left panel: previous/next day"),
        ("PgUp/PgDn", "Left panel: previous/next week"),
        ("↑/↓", "Right panel: same field in the previous/next period"),
        ("Tab/Shift-Tab", "Right panel: next/previous field"),
        ("Ctrl+↑/↓", "Right panel: previous/next day"),
        ("F1", "Help"),
        ("F2, Esc", "Back to Tracking"),
        ("Ctrl+Q", "Quit"),
    ];
    let lines: Vec<Line> = shortcuts
        .iter()
        .map(|(key, action)| Line::raw(format!(" {key:<15}{action}")))
        .collect();
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel_block(" Help ", true))
            .style(PANEL),
        area,
    );
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let text = format!(
        "Terminal too small: need at least {MIN_WIDTH}×{MIN_HEIGHT}, have {}×{}",
        area.width, area.height
    );
    frame.render_widget(Paragraph::new(text), area);
}
