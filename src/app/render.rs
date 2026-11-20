//! Drawing the application state in the style of Midnight Commander.

use jiff::civil::Date;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph},
};

use super::{App, Field, Panel, Popup, Screen};
use crate::domain::{
    Day, Minutes, Period, PeriodIndicator, PeriodType, TimeOfDay,
    calendar::{self, Separator},
};

const MIN_WIDTH: u16 = 120;
const MIN_HEIGHT: u16 = 24;
const LEFT_PANEL_WIDTH: u16 = 46;

// Left panel columns.
const DATE_WIDTH: usize = 10;
const WORKED_WIDTH: usize = 6;
const DAY_BALANCE_WIDTH: usize = 7;
const TOTAL_BALANCE_WIDTH: usize = 8;

// Right panel columns: where each starts in a period row.
const GAP: usize = 1;
const WARNING_WIDTH: usize = 2;
const TIME_WIDTH: usize = 4;
const DURATION_WIDTH: usize = 8;
const PROJECT_WIDTH: usize = 10;
const CHECKBOX_WIDTH: usize = 3;
const FROM_X: usize = WARNING_WIDTH + GAP;
const TO_X: usize = FROM_X + TIME_WIDTH + GAP;
const DURATION_X: usize = TO_X + TIME_WIDTH + GAP;
const PROJECT_X: usize = DURATION_X + DURATION_WIDTH + GAP;
const CHECKBOXES_X: usize = PROJECT_X + PROJECT_WIDTH;
const NOTES_X: usize = CHECKBOXES_X + Field::CHECKBOXES.len() * (GAP + CHECKBOX_WIDTH) + GAP;

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

const PANEL: Style = Style::new().fg(Color::Gray).bg(Color::Blue);
const HEADER: Style = Style::new().fg(Color::Yellow).bg(Color::Blue);
const ACTIVE_TITLE: Style = Style::new().fg(Color::Black).bg(Color::Gray);
const SELECTED: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
/// The selected day in an inactive left panel.
const INACTIVE_SELECTED: Style = Style::new().fg(Color::Cyan).bg(Color::Blue);
const FIELD: Style = Style::new().fg(Color::Black).bg(Color::Gray);
const FOCUSED_FIELD: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
const MESSAGE: Style = Style::new().fg(Color::Yellow).bg(Color::Blue);
/// Periods that failed the consistency check.
const HIGHLIGHTED: Style = Style::new().fg(Color::White).bg(Color::Red);
const POPUP: Style = Style::new().fg(Color::Black).bg(Color::Gray);
const POPUP_TITLE: Style = Style::new().fg(Color::Blue).bg(Color::Gray);
const SHADOW: Style = Style::new().bg(Color::Black);
const BUTTON: Style = Style::new().fg(Color::Black).bg(Color::White);
const SELECTED_BUTTON: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
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
        if let Some(popup) = &self.popup {
            render_popup(frame, popup);
        }
    }

    fn render_tracking(&mut self, frame: &mut Frame, area: Rect) {
        let [left, right] =
            Layout::horizontal([Constraint::Length(LEFT_PANEL_WIDTH), Constraint::Fill(1)])
                .areas(area);
        self.render_days(frame, left);
        self.render_periods(frame, right);
    }

    fn render_days(&mut self, frame: &mut Frame, area: Rect) {
        let title = format!(" Days: {} ", self.selected.year());
        let block = panel_block(&title, self.panel == Panel::Days);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [header, list] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        let header_text = day_columns("Date", "Worked", "Day", "Total", "");
        frame.render_widget(Paragraph::new(header_text).style(HEADER), header);

        let height = usize::from(list.height);
        let first = self.scroll_to_selected_day(height);
        let width = usize::from(list.width);
        let lines: Vec<Line> = list_rows(first)
            .take(height)
            .map(|row| match row {
                ListRow::Day(date) => self.day_line(date, width),
                ListRow::Separator(Separator::Year, date) => {
                    Line::raw(year_separator(width, calendar::next_weekday(date).year()))
                }
                ListRow::Separator(_, _) => Line::raw("─".repeat(width)),
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).style(PANEL), list);
    }

    fn day_line(&self, date: Date, width: usize) -> Line<'static> {
        let style = match (date == self.selected, self.panel) {
            (false, _) => PANEL,
            (true, Panel::Days) => SELECTED,
            (true, Panel::Periods) => INACTIVE_SELECTED,
        };
        Line::from(Span::styled(fit(&self.day_row_text(date), width), style))
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
        let warning = warning_emoji(timesheet.has_day_warning_on(date, self.today));
        day_columns(
            &calendar::format_dutch(date),
            &worked,
            &day_balance,
            &total_balance,
            &format!("{absence} {warning}"),
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
        let title = format!(" Periods: {} ", calendar::format_dutch_short(self.selected));
        let block = panel_block(&title, self.panel == Panel::Periods);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [header, list, message, actual_expected] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        let notes_width = usize::from(inner.width).saturating_sub(NOTES_X);

        let checkbox_headers: String = Field::CHECKBOXES
            .iter()
            .filter_map(|field| field.period_type())
            .map(|period_type| format!(" {} ", period_type_emoji(period_type)))
            .collect();
        let header_text = plain_period_row(
            ["", "From", "To", "Duration", "Project"],
            &checkbox_headers,
            "Notes",
        );
        frame.render_widget(Paragraph::new(header_text).style(HEADER), header);

        let day = self.timesheet.day(self.selected);
        let rows = period_rows(day);
        let height = usize::from(list.height);
        let first = self.scroll_to_focused_period(&rows, height);
        let active_input_text = match (self.focus, self.active_input.as_mut()) {
            (Some(focus), Some(input)) => {
                let (x, width) = text_field_geometry(focus.field, notes_width).unwrap_or_default();
                let (text, column) = input.visible(width);
                let row = row_of(&rows, focus.period)
                    .and_then(|row| row.checked_sub(first))
                    .filter(|&row| row < height);
                if let Some(row) = row {
                    frame.set_cursor_position((list.x + to_u16(x + column), list.y + to_u16(row)));
                }
                Some(text)
            }
            _ => None,
        };
        // Looked up again: the active input above needed `self` mutably.
        let day = self.timesheet.day(self.selected);
        let indicators = day.period_indicators(self.selected, self.today);
        let lines: Vec<Line> = rows
            .iter()
            .skip(first)
            .take(height)
            .map(|&row| match row {
                PeriodRow::Blank => Line::raw(""),
                PeriodRow::ImpliedBreak(after) => implied_break_line(day, after),
                PeriodRow::Period(index) => {
                    let mut line = self.period_line(
                        index,
                        &day.periods[index],
                        indicators[index],
                        notes_width,
                        active_input_text.as_deref(),
                    );
                    if self.inconsistent_periods.contains(&index) {
                        for span in &mut line.spans {
                            span.style = HIGHLIGHTED;
                        }
                    }
                    line
                }
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).style(PANEL), list);

        if let Some(text) = &self.message {
            frame.render_widget(Paragraph::new(format!(" {text}")).style(MESSAGE), message);
        }

        frame.render_widget(
            Paragraph::new(format!(" {}", actual_expected_text(day))).style(PANEL),
            actual_expected,
        );
    }

    fn period_line<'a>(
        &self,
        index: usize,
        period: &'a Period,
        indicator: Option<PeriodIndicator>,
        notes_width: usize,
        active_input_text: Option<&str>,
    ) -> Line<'a> {
        let text_field = |field: Field, text: &str, width: usize| match (
            self.is_focused(index, field),
            active_input_text,
        ) {
            (true, Some(active_input)) => Span::styled(fit(active_input, width), FOCUSED_FIELD),
            (true, None) => Span::styled(fit(text, width), FOCUSED_FIELD),
            (false, _) => Span::styled(fit(text, width), FIELD),
        };
        let gap = || Span::raw(" ");
        let mut spans = vec![
            Span::raw(indicator_emoji(indicator)),
            gap(),
            text_field(Field::From, &format_time(period.from), TIME_WIDTH),
            gap(),
            text_field(Field::To, &format_time(period.to), TIME_WIDTH),
            gap(),
            Span::raw(format!("{:<DURATION_WIDTH$}", format_duration(period))),
            gap(),
            text_field(
                Field::Project,
                period.project.as_deref().unwrap_or_default(),
                PROJECT_WIDTH,
            ),
        ];
        for field in Field::CHECKBOXES {
            let style = if self.is_focused(index, field) {
                FOCUSED_FIELD
            } else {
                FIELD
            };
            spans.extend([
                gap(),
                Span::styled(checkbox(field, period.period_type), style),
            ]);
        }
        spans.extend([
            gap(),
            text_field(
                Field::Notes,
                period.notes.as_deref().unwrap_or_default(),
                notes_width,
            ),
        ]);
        Line::from(spans)
    }

    fn is_focused(&self, period: usize, field: Field) -> bool {
        self.focus
            .is_some_and(|f| f.period == period && f.field == field)
    }

    /// Adjusts the first visible row so the first period that failed the
    /// consistency check, or else the focused period, is visible.
    fn scroll_to_focused_period(&mut self, rows: &[PeriodRow], height: usize) -> usize {
        let target = self
            .inconsistent_periods
            .iter()
            .min()
            .copied()
            .or(self.focus.map(|focus| focus.period))
            .and_then(|period| row_of(rows, period));
        if let Some(row) = target {
            if row < self.first_visible_period_row {
                self.first_visible_period_row = row;
            } else if height > 0 && row >= self.first_visible_period_row + height {
                self.first_visible_period_row = row + 1 - height;
            }
        }
        self.first_visible_period_row = self
            .first_visible_period_row
            .min(rows.len().saturating_sub(1));
        self.first_visible_period_row
    }
}

/// A row in the right panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PeriodRow {
    Period(usize),
    /// The implied break after the period with this index.
    ImpliedBreak(usize),
    Blank,
}

/// The row on which the period with this index is shown.
fn row_of(rows: &[PeriodRow], period: usize) -> Option<usize> {
    rows.iter()
        .position(|&row| row == PeriodRow::Period(period))
}

/// The right panel's rows: periods separated by empty rows, with implied
/// breaks as rows of their own.
fn period_rows(day: &Day) -> Vec<PeriodRow> {
    let mut rows = Vec::new();
    for index in 0..day.periods.len() {
        if index > 0 {
            rows.push(PeriodRow::Blank);
        }
        rows.push(PeriodRow::Period(index));
        if day.has_implied_break_after(index) {
            rows.extend([PeriodRow::Blank, PeriodRow::ImpliedBreak(index)]);
        }
    }
    rows
}

/// An implied break as plain text: from the end of one period to the start
/// of the next, with Break checked.
fn implied_break_line(day: &Day, after: usize) -> Line<'static> {
    let Some(implied) = day.implied_break_after(after) else {
        return Line::raw("");
    };
    let checkboxes: String = Field::CHECKBOXES
        .iter()
        .map(|&field| format!(" {}", checkbox(field, implied.period_type)))
        .collect();
    Line::raw(plain_period_row(
        [
            "",
            &format_time(implied.from),
            &format_time(implied.to),
            &format_duration(&implied),
            "",
        ],
        &checkboxes,
        "",
    ))
}

/// A period row as plain text, for the header and implied breaks:
/// `[warning, from, to, duration, project]`, the checkbox columns, and notes.
fn plain_period_row(columns: [&str; 5], checkboxes: &str, notes: &str) -> String {
    let [warning, from, to, duration, project] = columns;
    format!(
        "{warning:<WARNING_WIDTH$} {from:<TIME_WIDTH$} {to:<TIME_WIDTH$} {duration:<DURATION_WIDTH$} {project:<PROJECT_WIDTH$}{checkboxes} {notes}"
    )
}

/// The day warning in the left panel.
fn warning_emoji(warn: bool) -> &'static str {
    if warn { "⚠️" } else { "  " }
}

/// The indicator left of From.
fn indicator_emoji(indicator: Option<PeriodIndicator>) -> &'static str {
    match indicator {
        Some(PeriodIndicator::ConsistencyFailure) => "⚠️",
        Some(PeriodIndicator::PeriodWarning) => "❗",
        Some(PeriodIndicator::NonWorking(period_type)) => period_type_emoji(period_type),
        None => "  ",
    }
}

/// A checkbox's content: a check mark when checked; the box itself is its background.
fn checkbox(field: Field, period_type: PeriodType) -> &'static str {
    if field.period_type() == Some(period_type) {
        " ✓ "
    } else {
        "   "
    }
}

fn format_time(time: Option<TimeOfDay>) -> String {
    time.map(TimeOfDay::format_hhmm).unwrap_or_default()
}

fn format_duration(period: &Period) -> String {
    period
        .duration()
        .map(Minutes::format_duration)
        .unwrap_or_default()
}

/// Where a text field starts in a period row, and how wide it is; `None` for checkboxes.
fn text_field_geometry(field: Field, notes_width: usize) -> Option<(usize, usize)> {
    match field {
        Field::From => Some((FROM_X, TIME_WIDTH)),
        Field::To => Some((TO_X, TIME_WIDTH)),
        Field::Project => Some((PROJECT_X, PROJECT_WIDTH)),
        Field::Notes => Some((NOTES_X, notes_width)),
        Field::Break | Field::Vacation | Field::Sick | Field::Holiday => None,
    }
}

/// The left panel's columns.
fn day_columns(date: &str, worked: &str, day: &str, total: &str, indicators: &str) -> String {
    format!(
        " {date:<DATE_WIDTH$}{worked:>WORKED_WIDTH$} {day:>DAY_BALANCE_WIDTH$} {total:>TOTAL_BALANCE_WIDTH$}  {indicators}"
    )
}

/// A double line with the new year in the middle, e.g. `═══ 2027 ═══`.
fn year_separator(width: usize, year: i16) -> String {
    let label = format!(" {year} ");
    let left = width.saturating_sub(label.len()) / 2;
    let right = width.saturating_sub(left + label.len());
    format!("{}{label}{}", "═".repeat(left), "═".repeat(right))
}

fn to_u16(n: usize) -> u16 {
    u16::try_from(n).unwrap_or(u16::MAX)
}

const POPUP_MIN_WIDTH: usize = 40;
/// Space between the popup's edge and its frame: (columns, rows).
const POPUP_MARGIN: (u16, u16) = (2, 1);
/// How far the popup's shadow is offset: (columns, rows).
const SHADOW_OFFSET: (u16, u16) = (2, 1);
/// A button's padding on both sides plus the gap to the next button.
const BUTTON_EXTRA_WIDTH: usize = 4;

fn render_popup(frame: &mut Frame, popup: &Popup) {
    let buttons_width: usize = popup
        .buttons
        .iter()
        .map(|(label, _)| Span::raw(*label).width() + BUTTON_EXTRA_WIDTH)
        .sum();
    let title = format!(" {} ", popup.title);
    // The outer margin on both sides, and the frame with its padding.
    let chrome_width = 2 * usize::from(POPUP_MARGIN.0) + 4;
    let max_width = usize::from(frame.area().width.saturating_sub(SHADOW_OFFSET.0));
    let width = (Span::raw(popup.message.as_str()).width() + chrome_width)
        .max(buttons_width + chrome_width)
        .max(Span::raw(title.as_str()).width() + chrome_width)
        .max(POPUP_MIN_WIDTH)
        .min(max_width);
    let message = wrap(&popup.message, width.saturating_sub(chrome_width));
    // Margin, frame, message, blank line, buttons, frame, margin.
    let height = to_u16(message.len() + 6);
    let width = to_u16(width);
    let area = Rect {
        x: frame.area().width.saturating_sub(width) / 2,
        y: frame.area().height.saturating_sub(height) / 2,
        width,
        height,
    };
    let shadow = Rect {
        x: area.x + SHADOW_OFFSET.0,
        y: area.y + SHADOW_OFFSET.1,
        ..area
    }
    .intersection(frame.area());
    frame.render_widget(Clear, shadow);
    frame.render_widget(Block::new().style(SHADOW), shadow);
    frame.render_widget(Clear, area);
    frame.render_widget(Block::new().style(POPUP), area);

    let frame_area = Rect {
        x: area.x + POPUP_MARGIN.0,
        y: area.y + POPUP_MARGIN.1,
        width: area.width.saturating_sub(2 * POPUP_MARGIN.0),
        height: area.height.saturating_sub(2 * POPUP_MARGIN.1),
    };
    let block = Block::bordered()
        .title(Span::styled(title, POPUP_TITLE))
        .title_alignment(Alignment::Center)
        .padding(Padding::horizontal(1))
        .style(POPUP);
    let inner = block.inner(frame_area);
    frame.render_widget(block, frame_area);

    let mut buttons: Vec<Span> = Vec::new();
    for (i, (label, _)) in popup.buttons.iter().enumerate() {
        let style = if i == popup.selected {
            SELECTED_BUTTON
        } else {
            BUTTON
        };
        let mut chars = label.chars();
        let hotkey: String = chars.next().into_iter().collect();
        if i > 0 {
            buttons.push(Span::styled("  ", POPUP));
        }
        buttons.extend([
            Span::styled(" ", style),
            Span::styled(hotkey, style.fg(Color::Blue)),
            Span::styled(format!("{} ", chars.as_str()), style),
        ]);
    }
    let mut lines: Vec<Line> = message.into_iter().map(Line::raw).collect();
    lines.extend([Line::raw(""), Line::from(buttons).centered()]);
    frame.render_widget(Paragraph::new(lines).style(POPUP), inner);
}

/// Wraps `text` at spaces into lines of at most `width` cells.
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = vec![String::new()];
    for word in text.split(' ') {
        let line = lines.last_mut().expect("starts with one line");
        let needed = Span::raw(line.as_str()).width() + 1 + Span::raw(word).width();
        if line.is_empty() {
            line.push_str(word);
        } else if needed <= width {
            line.push(' ');
            line.push_str(word);
        } else {
            lines.push(word.to_owned());
        }
    }
    lines
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
    /// The separator after this day.
    Separator(Separator, Date),
}

/// The endless list of rows starting at day `first`.
fn list_rows(first: Date) -> impl Iterator<Item = ListRow> {
    std::iter::successors(Some(first), |&date| Some(calendar::next_weekday(date))).flat_map(
        |date| {
            let separator = calendar::separator_after(date);
            std::iter::once(ListRow::Day(date)).chain(
                (separator != Separator::None).then_some(ListRow::Separator(separator, date)),
            )
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
    let title_style = if active { ACTIVE_TITLE } else { PANEL };
    Block::bordered()
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
        ("←/→", "Right panel: previous/next field of the period"),
        ("Alt+↑/↓", "Right panel: previous/next day"),
        ("Alt+PgUp/PgDn", "Right panel: previous/next week"),
        (
            "Type",
            "Right panel: replace the field's value (Notes: add to it)",
        ),
        (
            "Enter",
            "Edit the focused field, or toggle a checkbox; while editing: accept",
        ),
        ("Alt+Enter", "Add a period after the last one"),
        ("Esc", "Reject input"),
        ("Alt+Del", "Delete the focused period"),
        ("Alt+b/v/s/h", "Toggle break/vacation/sick/holiday"),
        ("Alt+p/n", "Go to Project/Notes"),
        ("F1", "Help"),
        ("F2, Esc", "Back to Tracking"),
        ("Ctrl+C/Ctrl+V", "Right panel: copy/paste the focused field"),
        ("Ctrl+Z/Ctrl+Y", "Undo/redo"),
        ("Ctrl+S", "Save"),
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
