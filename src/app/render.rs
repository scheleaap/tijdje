//! Drawing the application state in the style of Midnight Commander.

use jiff::civil::Date;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Clear, Padding, Paragraph},
};

use super::{
    App, Dialog, Field, FieldFocus, Focus,
    active_input::ActiveInput,
    help::{HelpLine, HelpWindow, help_lines},
};
use crate::domain::{
    Day, DayIndicator, Extent, Minutes, Period, PeriodIndicator, PeriodType, TimeOfDay,
    WORK_UNTIL_TARGET,
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
/// The part-of-day field covers From, the gap and To.
const PART_OF_DAY_WIDTH: usize = TIME_WIDTH + GAP + TIME_WIDTH;
/// Durations are right-aligned in this width, so `42m` lines up with `1h55m`.
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
/// Global notifications: black on cyan, like the selection bar.
const NOTIFICATION: Style = SELECTED;
/// The values on the bottom line.
const BOTTOM_LINE_VALUE: Style = SELECTED;
/// Periods that failed the consistency check.
const HIGHLIGHTED: Style = Style::new().fg(Color::White).bg(Color::Red);
const DIALOG: Style = Style::new().fg(Color::Black).bg(Color::Gray);
const DIALOG_TITLE: Style = Style::new().fg(Color::Blue).bg(Color::Gray);
const SHADOW: Style = Style::new().bg(Color::Black);
const BUTTON: Style = Style::new().fg(Color::Black).bg(Color::White);
const SELECTED_BUTTON: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
const KEY_NUMBER: Style = Style::new().fg(Color::White).bg(Color::Black);
const KEY_LABEL: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
/// The Help window, in the colors of Midnight Commander's user menu.
const HELP: Style = Style::new().fg(Color::White).bg(Color::Cyan);
const HELP_TITLE: Style = Style::new().fg(Color::Yellow).bg(Color::Cyan);
/// The Help window's group headings, yellow like the title.
const HELP_HEADING: Style = HELP_TITLE;
/// The width of the Help window's key column, including the gap to the action.
const HELP_KEY_WIDTH: usize = 30;
/// The Help window's frame: one cell on each side.
const HELP_FRAME_SIZE: u16 = 2;
/// In the Help window's bottom frame while not all lines fit.
const HELP_SCROLL_HINT: &str = " ↑↓ PgUp/PgDn scroll ";
/// The right panel's padding inside its frame for the day note and the
/// bottom line; the Notes field keeps the same right padding.
const PANEL_PADDING: Padding = Padding::horizontal(1);

impl App {
    pub fn render(&mut self, frame: &mut Frame) {
        let area = frame.area();
        if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
            render_too_small(frame, area);
            return;
        }
        let [main, keys] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(area);
        self.render_tracking(frame, main);
        if let Some(help) = &mut self.help {
            render_help_window(frame, main, help);
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
        let indicator = day_indicator_emoji(timesheet.day_indicator_on(date, self.today));
        day_columns(
            &calendar::format_dutch(date),
            &worked,
            &day_balance,
            &total_balance,
            indicator,
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
        let areas = PeriodPanelAreas::of(block.inner(area));
        frame.render_widget(block, area);
        render_period_header(frame, areas.header);
        frame.render_widget(
            Paragraph::new("Day note").style(HEADER),
            areas.day_note_header,
        );

        let rows = period_rows(&self.shown_day());
        let first = self.scroll_to_focused_period(&rows, usize::from(areas.list.height));
        let active_input_text = self.place_active_input(frame, &areas, &rows, first);
        // Looked up only now: the active input above needed `self` mutably.
        let day = self.shown_day();
        frame.render_widget(
            self.day_note_text(
                &day,
                usize::from(areas.day_note.width),
                active_input_text.as_ref(),
            ),
            areas.day_note,
        );
        let lines = self.period_list_lines(&day, &rows, first, &areas, active_input_text.as_ref());
        frame.render_widget(Paragraph::new(lines).style(PANEL), areas.list);
        frame.render_widget(
            Paragraph::new(bottom_line(&day))
                .block(padded_block())
                .style(PANEL),
            areas.bottom_line,
        );
    }

    /// The active input as it is shown in the focused input field, with the
    /// terminal cursor placed in it; `None` while not editing.
    fn place_active_input(
        &mut self,
        frame: &mut Frame,
        areas: &PeriodPanelAreas,
        rows: &[PeriodRow],
        first: usize,
    ) -> Option<ActiveInputText> {
        let suggestion = self.project_suggestion();
        let focus = self.focus;
        let input = self.active_input.as_mut()?;
        let (x, y, width, suggestion) = match focus {
            Focus::DayNote => (
                areas.day_note.x,
                Some(areas.day_note.y),
                usize::from(areas.day_note.width),
                None,
            ),
            Focus::Period(focus) => {
                let (x, width) =
                    text_field_geometry(focus.field, areas.notes_width).unwrap_or_default();
                let height = usize::from(areas.list.height);
                let row = row_of(rows, focus.period)
                    .and_then(|row| row.checked_sub(first))
                    .filter(|&row| row < height);
                (
                    areas.list.x + to_u16(x),
                    row.map(|row| areas.list.y + to_u16(row)),
                    width,
                    suggestion,
                )
            }
        };
        let (text, column) = input.visible(width);
        if let Some(y) = y {
            frame.set_cursor_position((x + to_u16(column), y));
        }
        Some(ActiveInputText::new(text, input.len(), suggestion, width))
    }

    /// The visible rows of the period list, from row `first` on: periods,
    /// implied breaks and the empty rows between them.
    fn period_list_lines<'a>(
        &self,
        day: &'a Day,
        rows: &[PeriodRow],
        first: usize,
        areas: &PeriodPanelAreas,
        active_input_text: Option<&ActiveInputText>,
    ) -> Vec<Line<'a>> {
        let indicators = day.period_indicators(self.selected, self.today);
        rows.iter()
            .skip(first)
            .take(usize::from(areas.list.height))
            .map(|&row| match row {
                PeriodRow::Blank => Line::raw(""),
                PeriodRow::ImpliedBreak(after) => implied_break_line(day, after),
                PeriodRow::Period(index) => {
                    let mut line = self.period_line(
                        day,
                        index,
                        indicators[index],
                        areas.notes_width,
                        active_input_text,
                    );
                    if self.inconsistent_periods.contains(&index) {
                        for span in &mut line.spans {
                            span.style = HIGHLIGHTED;
                        }
                    }
                    line
                }
            })
            .collect()
    }

    /// The row of the period at `index` of `day`.
    fn period_line<'a>(
        &self,
        day: &'a Day,
        index: usize,
        indicator: Option<PeriodIndicator>,
        notes_width: usize,
        active_input_text: Option<&ActiveInputText>,
    ) -> Line<'a> {
        let period = &day.periods[index];
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
        match period.extent {
            Extent::PartOfDay(part_of_day) => spans.extend(text_field(
                Field::PartOfDay,
                part_of_day.label(),
                PART_OF_DAY_WIDTH,
            )),
            Extent::FromTo { from, to } => {
                spans.extend(text_field(Field::From, &format_time(from), TIME_WIDTH));
                spans.push(gap());
                spans.extend(text_field(Field::To, &format_time(to), TIME_WIDTH));
            }
        }
        spans.extend([
            gap(),
            Span::raw(format!("{:<DURATION_WIDTH$}", format_duration(day, period))),
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
    /// while it is being edited.
    fn day_note_text(
        &self,
        day: &Day,
        width: usize,
        active_input_text: Option<&ActiveInputText>,
    ) -> Span<'static> {
        let focused = self.focus == Focus::DayNote;
        let (text, style) = match (active_input_text.filter(|_| focused), &day.day_note) {
            (Some(active_input), _) => (active_input.text.as_str(), FOCUSED_FIELD),
            (None, day_note) => (
                day_note.as_deref().unwrap_or_default(),
                if focused { FOCUSED_FIELD } else { FIELD },
            ),
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
            // The day note is always visible below the list, so focusing it
            // doesn't scroll.
            .or(self.focus.period());
        if let Some(row) = target.and_then(|target| row_of(rows, target)) {
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

/// The parts of the right panel inside its frame, from top to bottom.
struct PeriodPanelAreas {
    header: Rect,
    list: Rect,
    day_note_header: Rect,
    day_note: Rect,
    bottom_line: Rect,
    /// The width of the Notes input field, which ends at the right padding.
    notes_width: usize,
}

impl PeriodPanelAreas {
    /// Lays out the panel's `inner` area: the period header and list, then
    /// the day note's header and input field, an empty line and the bottom
    /// line. The day note lines up with the bottom line's padding.
    fn of(inner: Rect) -> Self {
        let [header, list, day_note_header, day_note, _, bottom_line] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .areas(inner);
        Self {
            header,
            list,
            day_note_header: padded_block().inner(day_note_header),
            day_note: padded_block().inner(day_note),
            bottom_line,
            notes_width: usize::from(inner.width)
                .saturating_sub(NOTES_X + usize::from(PANEL_PADDING.right)),
        }
    }
}

/// A frameless block with the right panel's padding, shared by the day note
/// and the bottom line.
fn padded_block() -> Block<'static> {
    Block::new().padding(PANEL_PADDING)
}

/// The period column headers: From, To, Duration, Project, the checkbox
/// emoji and Notes.
fn render_period_header(frame: &mut Frame, area: Rect) {
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
    frame.render_widget(Paragraph::new(header_text).style(HEADER), area);
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
            &format_time(implied.from()),
            &format_time(implied.to()),
            &format_duration(day, &implied),
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

/// The day indicator right after the total balance.
fn day_indicator_emoji(indicator: Option<DayIndicator>) -> &'static str {
    match indicator {
        Some(DayIndicator::DayWarning) => "⚠️",
        Some(DayIndicator::Absence(absence)) => period_type_emoji(absence.into()),
        None => "  ",
    }
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

/// The duration of `period` on `day`, right-aligned with the other
/// durations; empty while it has no duration.
fn format_duration(day: &Day, period: &Period) -> String {
    day.duration_of(period)
        .map(|duration| format!("{:>DURATION_WIDTH$}", duration.format_duration()))
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
        Field::PartOfDay
        | Field::Break
        | Field::Free
        | Field::Vacation
        | Field::Sick
        | Field::Holiday => None,
    }
}

/// The left panel's columns.
fn day_columns(date: &str, worked: &str, day: &str, total: &str, indicator: &str) -> String {
    format!(
        " {date:<DATE_WIDTH$}{worked:>WORKED_WIDTH$} {day:>DAY_BALANCE_WIDTH$} {total:>TOTAL_BALANCE_WIDTH$}  {indicator}"
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
    render_shadow(frame, area);
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
    // A margin in the panel's colors keeps it apart from the cyan values on
    // the bottom line.
    let margin = area.outer(Margin::new(1, 0)).intersection(main);
    frame.render_widget(Clear, margin);
    frame.render_widget(Block::new().style(PANEL), margin);
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

/// The bottom line: worked time against the actual expected working time,
/// and until when to work for 8h, with the values on a cyan background.
fn bottom_line(day: &Day) -> Line<'static> {
    let work_until = day
        .work_until()
        .map_or_else(|| "–".to_owned(), TimeOfDay::format_with_colon);
    Line::from(vec![
        Span::raw("Worked "),
        Span::styled(day.worked_time().format_duration(), BOTTOM_LINE_VALUE),
        Span::raw(" of "),
        Span::styled(
            day.actual_expected_working_time().format_duration(),
            BOTTOM_LINE_VALUE,
        ),
        Span::raw(" | Work until "),
        Span::styled(work_until, BOTTOM_LINE_VALUE),
        Span::raw(" for "),
        Span::styled(format_target(), BOTTOM_LINE_VALUE),
    ])
}

/// The worked time "Work until" aims for, like `8h`: without `00m`.
fn format_target() -> String {
    let target = WORK_UNTIL_TARGET.format_duration();
    target
        .strip_suffix("00m")
        .map_or_else(|| target.clone(), str::to_owned)
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

/// The function-key bar: ten slots stretched evenly over the full width.
fn render_function_keys(frame: &mut Frame, area: Rect) {
    let labels = ["Help", "Track", "", "", "", "", "", "", "", ""];
    let slots = Layout::horizontal(labels.map(|_| Constraint::Fill(1))).split(area);
    labels
        .iter()
        .zip(slots.iter())
        .enumerate()
        .for_each(|(i, (label, &slot))| {
            let line = Line::from(vec![
                Span::styled(format!("{:>2}", i + 1), KEY_NUMBER),
                Span::raw(*label),
            ]);
            frame.render_widget(Paragraph::new(line).style(KEY_LABEL), slot);
        });
}

/// Draws a black shadow below and to the right of `area`.
fn render_shadow(frame: &mut Frame, area: Rect) {
    let shadow = Rect {
        x: area.x + SHADOW_OFFSET.0,
        y: area.y + SHADOW_OFFSET.1,
        ..area
    }
    .intersection(frame.area());
    frame.render_widget(Clear, shadow);
    frame.render_widget(Block::new().style(SHADOW), shadow);
}

/// Draws the Help window centered over `area`, scrolled as far as `help`
/// says. If not all lines fit, the bottom frame says how to scroll.
fn render_help_window(frame: &mut Frame, area: Rect, help: &mut HelpWindow) {
    let lines: Vec<Line> = help_lines().into_iter().map(help_line).collect();
    let window = help_window_area(area, &lines);
    let shown_lines = usize::from(window.height.saturating_sub(HELP_FRAME_SIZE));
    help.fit(lines.len(), shown_lines);
    let block = Block::bordered()
        .title(Span::styled(" Help ", HELP_TITLE))
        .title_alignment(Alignment::Center)
        .padding(Padding::horizontal(1))
        .style(HELP);
    let block = if lines.len() > shown_lines {
        block.title_bottom(Line::styled(HELP_SCROLL_HINT, HELP_TITLE).centered())
    } else {
        block
    };
    render_shadow(frame, window);
    frame.render_widget(Clear, window);
    frame.render_widget(
        Paragraph::new(lines)
            .block(block)
            .scroll((to_u16(help.first_line()), 0)),
        window,
    );
}

/// Where the Help window goes, centered over `area`: as large as `lines`
/// with the frame and padding need, but leaving room for the shadow.
fn help_window_area(area: Rect, lines: &[Line]) -> Rect {
    let content_width = lines.iter().map(Line::width).max().unwrap_or_default();
    // The frame and the padding on both sides.
    let chrome_width = HELP_FRAME_SIZE + 2;
    let width = (to_u16(content_width) + chrome_width).min(area.width - SHADOW_OFFSET.0);
    // One row of margin above the window, and one below it for the shadow.
    let height = (to_u16(lines.len()) + HELP_FRAME_SIZE).min(area.height - 2 * SHADOW_OFFSET.1);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// A line of the Help window: a heading, or a shortcut's key and action in
/// two columns.
fn help_line(line: HelpLine) -> Line<'static> {
    match line {
        HelpLine::Heading(heading) => Line::styled(heading, HELP_HEADING),
        HelpLine::Shortcut { key, action } => {
            Line::raw(format!("{}{action}", fit(key, HELP_KEY_WIDTH)))
        }
        HelpLine::Blank => Line::default(),
    }
}

fn render_too_small(frame: &mut Frame, area: Rect) {
    let text = format!(
        "Terminal too small: need at least {MIN_WIDTH}×{MIN_HEIGHT}, have {}×{}",
        area.width, area.height
    );
    frame.render_widget(Paragraph::new(text), area);
}
