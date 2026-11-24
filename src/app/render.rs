//! Drawing the application state in the style of Midnight Commander.

use jiff::civil::Date;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph},
};

use super::{App, Dialog, Field, FieldFocus, Focus, Screen, active_input::ActiveInput};
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
        PeriodType::Free => "🆓",
        PeriodType::Vacation => "🏖️",
        PeriodType::Sick => "🤒",
        PeriodType::Holiday => "🎄",
        PeriodType::Work => "  ",
    }
}

const PANEL: Style = Style::new().fg(Color::Gray).bg(Color::Blue);
const HEADER: Style = Style::new().fg(Color::Yellow).bg(Color::Blue);
const SELECTED: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
const FIELD: Style = Style::new().fg(Color::Black).bg(Color::Gray);
const FOCUSED_FIELD: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
/// The rest of a suggested project after the active input.
const GHOST_TEXT: Style = FOCUSED_FIELD.add_modifier(Modifier::DIM);
/// A hint in an empty input field, such as the day note's.
const PLACEHOLDER: Style = Style::new().fg(Color::DarkGray).bg(Color::Gray);
const DAY_NOTE_PLACEHOLDER: &str = "Day note";
/// Global notifications: black on cyan, like the selection bar.
const NOTIFICATION: Style = SELECTED;
/// Periods that failed the consistency check.
const HIGHLIGHTED: Style = Style::new().fg(Color::White).bg(Color::Red);
const DIALOG: Style = Style::new().fg(Color::Black).bg(Color::Gray);
const DIALOG_TITLE: Style = Style::new().fg(Color::Blue).bg(Color::Gray);
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
        if let Some(text) = &self.notification {
            render_notification(frame, main, text);
        }
        if let Some(dialog) = &self.dialog {
            render_dialog(frame, dialog, Some(dialog.selected), None);
        }
        if let Some(go_to_date) = &mut self.go_to_date {
            let selected = go_to_date.shown_selected_button();
            let input = DialogInput {
                input: &mut go_to_date.input,
                focused: go_to_date.input_focused,
            };
            render_dialog(frame, &go_to_date.dialog, selected, Some(input));
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
        let block = panel_block(&title);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [header, list] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        let header_text = day_columns("Date", "Worked", "Day", "Total", "");
        frame.render_widget(Paragraph::new(header_text).style(HEADER), header);

        let height = usize::from(list.height);
        let first = self.scroll_to_selected_day(height);
        let width = usize::from(list.width);
        let rows: Vec<ListRow> = list_rows(first).take(height).collect();
        self.last_visible_day = rows.iter().rev().find_map(|&row| match row {
            ListRow::Day(date) => Some(date),
            ListRow::Separator(_, _) => None,
        });
        let lines: Vec<Line> = rows
            .into_iter()
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
        let style = if date == self.selected {
            SELECTED
        } else {
            PANEL
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
        let warning = day_warning_emoji(timesheet.has_day_warning_on(date, self.today));
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
        let block = panel_block(&title);
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [day_note, header, list, actual_expected] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Fill(1),
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
            None,
            ["From", "To", "Duration", "Project"],
            &checkbox_headers,
            "Notes",
        );
        frame.render_widget(Paragraph::new(header_text).style(HEADER), header);

        let rows = period_rows(&self.shown_day());
        let height = usize::from(list.height);
        let first = self.scroll_to_focused_period(&rows, height);
        let suggestion = self.project_suggestion();
        let day_note_width = usize::from(day_note.width);
        let active_input_text = match (self.active_input.as_mut(), self.focus) {
            (Some(input), Focus::DayNote) => {
                let (text, column) = input.visible(day_note_width);
                frame.set_cursor_position((day_note.x + to_u16(column), day_note.y));
                Some(ActiveInputText::new(
                    text,
                    input.len(),
                    None,
                    day_note_width,
                ))
            }
            (Some(input), Focus::Period(focus)) => {
                let (x, width) = text_field_geometry(focus.field, notes_width).unwrap_or_default();
                let (text, column) = input.visible(width);
                let row = row_of(&rows, focus.period)
                    .and_then(|row| row.checked_sub(first))
                    .filter(|&row| row < height);
                if let Some(row) = row {
                    frame.set_cursor_position((list.x + to_u16(x + column), list.y + to_u16(row)));
                }
                Some(ActiveInputText::new(text, input.len(), suggestion, width))
            }
            (None, _) => None,
        };
        // Looked up only now: the active input above needed `self` mutably.
        let day = self.shown_day();
        frame.render_widget(
            self.day_note_text(&day, day_note_width, active_input_text.as_ref()),
            day_note,
        );
        let indicators = day.period_indicators(self.selected, self.today);
        let lines: Vec<Line> = rows
            .iter()
            .skip(first)
            .take(height)
            .map(|&row| match row {
                PeriodRow::Blank => Line::raw(""),
                PeriodRow::ImpliedBreak(after) => implied_break_line(&day, after),
                PeriodRow::Period(index) => {
                    let mut line = self.period_line(
                        index,
                        &day.periods[index],
                        indicators[index],
                        notes_width,
                        active_input_text.as_ref(),
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

        frame.render_widget(
            Paragraph::new(format!(" {}", actual_expected_text(&day))).style(PANEL),
            actual_expected,
        );
    }

    fn period_line<'a>(
        &self,
        index: usize,
        period: &'a Period,
        indicator: Option<PeriodIndicator>,
        notes_width: usize,
        active_input_text: Option<&ActiveInputText>,
    ) -> Line<'a> {
        let text_field = |field: Field, text: &str, width: usize| match (
            self.is_focused(index, field),
            active_input_text,
        ) {
            (true, Some(active_input)) => active_input.spans(width),
            (true, None) => vec![Span::styled(fit(text, width), FOCUSED_FIELD)],
            (false, _) => vec![Span::styled(fit(text, width), FIELD)],
        };
        let gap = || Span::raw(" ");
        let mut spans = vec![Span::raw(indicator_emoji(indicator)), gap()];
        spans.extend(text_field(
            Field::From,
            &format_time(period.from),
            TIME_WIDTH,
        ));
        spans.push(gap());
        spans.extend(text_field(Field::To, &format_time(period.to), TIME_WIDTH));
        spans.extend([
            gap(),
            Span::raw(format!("{:<DURATION_WIDTH$}", format_duration(period))),
            gap(),
        ]);
        spans.extend(text_field(
            Field::Project,
            period.project.as_deref().unwrap_or_default(),
            PROJECT_WIDTH,
        ));
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
        spans.push(gap());
        spans.extend(text_field(
            Field::Notes,
            period.notes.as_deref().unwrap_or_default(),
            notes_width,
        ));
        Line::from(spans)
    }

    /// The day note as a full-width input field, showing the active input
    /// while it is being edited, and a dimmed placeholder while it is empty
    /// and doesn't have focus.
    fn day_note_text(
        &self,
        day: &Day,
        width: usize,
        active_input_text: Option<&ActiveInputText>,
    ) -> Span<'static> {
        let focused = self.focus == Focus::DayNote;
        let (text, style) = match (active_input_text.filter(|_| focused), &day.day_note) {
            (Some(active_input), _) => (active_input.text.as_str(), FOCUSED_FIELD),
            (None, Some(day_note)) if focused => (day_note.as_str(), FOCUSED_FIELD),
            (None, Some(day_note)) => (day_note.as_str(), FIELD),
            (None, None) if focused => ("", FOCUSED_FIELD),
            (None, None) => (DAY_NOTE_PLACEHOLDER, PLACEHOLDER),
        };
        Span::styled(fit(text, width), style)
    }

    fn is_focused(&self, period: usize, field: Field) -> bool {
        self.focus == Focus::Period(FieldFocus { period, field })
    }

    /// Adjusts the first visible row so the first period that failed the
    /// consistency check, or else the focused period, is visible.
    fn scroll_to_focused_period(&mut self, rows: &[PeriodRow], height: usize) -> usize {
        let target = self
            .inconsistent_periods
            .iter()
            .min()
            .copied()
            .or(self.focus.period())
            // The day note is above the first period.
            .unwrap_or(0);
        let target = row_of(rows, target);
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
/// of the next, with the break indicator and Break checked.
fn implied_break_line(day: &Day, after: usize) -> Line<'static> {
    let Some(implied) = day.implied_break_after(after) else {
        return Line::raw("");
    };
    let checkboxes: String = Field::CHECKBOXES
        .iter()
        .map(|&field| format!(" {}", checkbox(field, implied.period_type)))
        .collect();
    Line::raw(plain_period_row(
        Some(PeriodIndicator::NonWorking(implied.period_type)),
        [
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
/// the indicator, `[from, to, duration, project]`, the checkbox columns, and notes.
fn plain_period_row(
    indicator: Option<PeriodIndicator>,
    columns: [&str; 4],
    checkboxes: &str,
    notes: &str,
) -> String {
    let [from, to, duration, project] = columns;
    // Already `WARNING_WIDTH` cells wide; `{:<}` would pad by characters.
    let indicator = indicator_emoji(indicator);
    format!(
        "{indicator} {from:<TIME_WIDTH$} {to:<TIME_WIDTH$} {duration:<DURATION_WIDTH$} {project:<PROJECT_WIDTH$}{checkboxes} {notes}"
    )
}

/// The day warning in the left panel.
fn day_warning_emoji(warn: bool) -> &'static str {
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

/// The visible part of the active input, and of the ghost text after it.
struct ActiveInputText {
    text: String,
    ghost_text: String,
}

impl ActiveInputText {
    /// `text`, the visible part of an active input of `typed_len` characters
    /// in a field `width` cells wide, followed by as much of the rest of
    /// `suggestion` as fits.
    fn new(text: String, typed_len: usize, suggestion: Option<String>, width: usize) -> Self {
        let rest: String = suggestion
            .map(|project| project.chars().skip(typed_len).collect())
            .unwrap_or_default();
        let room = width.saturating_sub(Span::raw(&text).width());
        Self {
            ghost_text: cut_to_width(&rest, room),
            text,
        }
    }

    /// The active input followed by the dimmed ghost text, filling `width` cells.
    fn spans(&self, width: usize) -> Vec<Span<'static>> {
        let used = Span::raw(&self.text).width() + Span::raw(&self.ghost_text).width();
        vec![
            Span::styled(self.text.clone(), FOCUSED_FIELD),
            Span::styled(self.ghost_text.clone(), GHOST_TEXT),
            Span::styled(fit("", width.saturating_sub(used)), FOCUSED_FIELD),
        ]
    }
}

/// The longest start of `text` that fits in `width` cells.
fn cut_to_width(text: &str, width: usize) -> String {
    text.chars()
        .scan(0, |used, c| {
            *used += Span::raw(c.to_string()).width();
            Some((*used, c))
        })
        .take_while(|&(used, _)| used <= width)
        .map(|(_, c)| c)
        .collect()
}

/// Where an input field starts in a period row, and how wide it is; `None` for checkboxes.
fn text_field_geometry(field: Field, notes_width: usize) -> Option<(usize, usize)> {
    match field {
        Field::From => Some((FROM_X, TIME_WIDTH)),
        Field::To => Some((TO_X, TIME_WIDTH)),
        Field::Project => Some((PROJECT_X, PROJECT_WIDTH)),
        Field::Notes => Some((NOTES_X, notes_width)),
        Field::Break | Field::Free | Field::Vacation | Field::Sick | Field::Holiday => None,
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

const DIALOG_MIN_WIDTH: usize = 40;
/// Space between the dialog's edge and its frame: (columns, rows).
const DIALOG_MARGIN: (u16, u16) = (2, 1);
/// How far the dialog's shadow is offset: (columns, rows).
const SHADOW_OFFSET: (u16, u16) = (2, 1);
/// A button's padding on both sides plus the gap to the next button.
const BUTTON_EXTRA_WIDTH: usize = 4;

/// A dialog's input field, shown below its message.
struct DialogInput<'a> {
    input: &'a mut ActiveInput,
    focused: bool,
}

/// Draws `dialog` with button `selected` highlighted (none if `None`), and
/// `input` as a full-width input field below the message, with the cursor
/// in it while it has focus.
fn render_dialog<A>(
    frame: &mut Frame,
    dialog: &Dialog<A>,
    selected: Option<usize>,
    input: Option<DialogInput>,
) {
    let buttons_width: usize = dialog
        .buttons
        .iter()
        .map(|(label, _)| Span::raw(*label).width() + BUTTON_EXTRA_WIDTH)
        .sum();
    let title = format!(" {} ", dialog.title);
    // The outer margin on both sides, and the frame with its padding.
    let chrome_width = 2 * usize::from(DIALOG_MARGIN.0) + 4;
    let max_width = usize::from(frame.area().width.saturating_sub(SHADOW_OFFSET.0));
    let width = (Span::raw(dialog.message.as_str()).width() + chrome_width)
        .max(buttons_width + chrome_width)
        .max(Span::raw(title.as_str()).width() + chrome_width)
        .max(DIALOG_MIN_WIDTH)
        .min(max_width);
    let message = wrap(&dialog.message, width.saturating_sub(chrome_width));
    let input_rows = usize::from(input.is_some());
    // Margin, frame, message, input field, blank line, buttons, frame, margin.
    let height = to_u16(message.len() + input_rows + 6);
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
    frame.render_widget(Block::new().style(DIALOG), area);

    let frame_area = Rect {
        x: area.x + DIALOG_MARGIN.0,
        y: area.y + DIALOG_MARGIN.1,
        width: area.width.saturating_sub(2 * DIALOG_MARGIN.0),
        height: area.height.saturating_sub(2 * DIALOG_MARGIN.1),
    };
    let block = Block::bordered()
        .title(Span::styled(title, DIALOG_TITLE))
        .title_alignment(Alignment::Center)
        .padding(Padding::horizontal(1))
        .style(DIALOG);
    let inner = block.inner(frame_area);
    frame.render_widget(block, frame_area);

    let mut buttons: Vec<Span> = Vec::new();
    for (i, (label, _)) in dialog.buttons.iter().enumerate() {
        let style = if Some(i) == selected {
            SELECTED_BUTTON
        } else {
            BUTTON
        };
        let mut chars = label.chars();
        let hotkey: String = chars.next().into_iter().collect();
        if i > 0 {
            buttons.push(Span::styled("  ", DIALOG));
        }
        buttons.extend([
            Span::styled(" ", style),
            Span::styled(hotkey, style.fg(Color::Blue)),
            Span::styled(format!("{} ", chars.as_str()), style),
        ]);
    }
    let message_rows = message.len();
    let mut lines: Vec<Line> = message.into_iter().map(Line::raw).collect();
    if let Some(DialogInput { input, focused }) = input {
        let width = usize::from(inner.width);
        // One cell less, so the cursor at the end stays inside the field.
        let (text, column) = input.visible(width.saturating_sub(1));
        let style = if focused { FOCUSED_FIELD } else { FIELD };
        lines.push(Line::from(Span::styled(fit(&text, width), style)));
        if focused {
            frame.set_cursor_position((inner.x + to_u16(column), inner.y + to_u16(message_rows)));
        }
    }
    lines.extend([Line::raw(""), Line::from(buttons).centered()]);
    frame.render_widget(Paragraph::new(lines).style(DIALOG), inner);
}

/// A global notification: `text` centered across the screen, floating over
/// the row just above the bottom border of the panels in `main`.
fn render_notification(frame: &mut Frame, main: Rect, text: &str) {
    let width = (Span::raw(text).width() + 2).min(usize::from(main.width));
    let line = format!(" {} ", fit(text, width.saturating_sub(2)));
    let width = to_u16(width);
    let area = Rect {
        x: main.x + main.width.saturating_sub(width) / 2,
        y: main.bottom().saturating_sub(2),
        width,
        height: 1,
    };
    frame.render_widget(Clear, area);
    frame.render_widget(Paragraph::new(line).style(NOTIFICATION), area);
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

/// A panel frame; its title has no highlight.
fn panel_block(title: &str) -> Block<'_> {
    Block::bordered().title(title).style(PANEL)
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
        ("Ctrl+↑/↓", "Previous/next day"),
        ("Ctrl+PgUp/PgDn", "Previous/next week"),
        ("Ctrl+G", "Go to date"),
        (
            "←/→/↑/↓",
            "Field left/right/above/below; editing a project, → at the end takes the suggestion",
        ),
        ("Tab/Shift-Tab", "Next/previous field"),
        (
            "Ctrl+Enter",
            "From of the next period; in the last one: add a period",
        ),
        ("Type", "Replace the field's value"),
        (
            "Enter",
            "Edit the focused field, or toggle a checkbox; while editing: accept",
        ),
        ("Esc", "Reject input"),
        ("Del", "Clear the field, or uncheck the checkbox"),
        (
            "Home/End, Ctrl+A/E",
            "While editing: start/end of the input",
        ),
        ("Ctrl+←/→", "While editing: previous/next word"),
        (
            "Ctrl+Backspace/Ctrl+W",
            "While editing: delete the previous word",
        ),
        ("Ctrl+U/Ctrl+K", "While editing: delete to the start/end"),
        ("Ctrl+Del", "Delete the focused period"),
        ("Alt+b/f/s/v/h", "Toggle break/free/sick/vacation/holiday"),
        ("F1", "Help"),
        ("F2, Esc", "Back to Tracking"),
        ("Ctrl+C/Ctrl+V", "Copy/paste the focused field"),
        ("Ctrl+Z/Ctrl+Y", "Undo/redo"),
        // One line, so the list still fits a 24-row terminal until the help
        // dialog scrolls (ticket 25).
        ("Ctrl+S/Ctrl+Q", "Save/quit"),
    ];
    let lines: Vec<Line> = shortcuts
        .iter()
        .map(|(key, action)| Line::raw(format!(" {key:<23}{action}")))
        .collect();
    frame.render_widget(
        Paragraph::new(lines)
            .block(panel_block(" Help "))
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
