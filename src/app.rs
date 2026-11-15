//! Interactive application state and rendering.

use jiff::civil::Date;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
};

use crate::domain::{
    Timesheet,
    calendar::{self, Separator},
};

const MIN_WIDTH: u16 = 100;
const MIN_HEIGHT: u16 = 24;
const LEFT_PANEL_WIDTH: u16 = 46;

const PANEL: Style = Style::new().fg(Color::Cyan).bg(Color::Blue);
const SELECTED: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
const KEY_NUMBER: Style = Style::new().fg(Color::White).bg(Color::Black);
const KEY_LABEL: Style = Style::new().fg(Color::Black).bg(Color::Cyan);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Tracking,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Continue,
    Quit,
}

pub struct App {
    timesheet: Timesheet,
    screen: Screen,
    selected: Date,
    /// `None` until the first render.
    first_visible_day: Option<Date>,
}

impl App {
    pub fn new(timesheet: Timesheet, today: Date) -> Self {
        Self {
            timesheet,
            screen: Screen::Tracking,
            selected: calendar::start_day(today),
            first_visible_day: None,
        }
    }

    pub fn screen(&self) -> Screen {
        self.screen
    }

    pub fn selected_day(&self) -> Date {
        self.selected
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Outcome {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('q') {
            return Outcome::Quit;
        }
        match (self.screen, key.code) {
            (_, KeyCode::F(1)) => self.screen = Screen::Help,
            (_, KeyCode::F(2)) | (Screen::Help, KeyCode::Esc) => self.screen = Screen::Tracking,
            (Screen::Tracking, KeyCode::Up) => {
                self.selected = calendar::previous_weekday(self.selected)
            }
            (Screen::Tracking, KeyCode::Down) => {
                self.selected = calendar::next_weekday(self.selected)
            }
            (Screen::Tracking, KeyCode::PageUp) => {
                self.selected = calendar::previous_week(self.selected)
            }
            (Screen::Tracking, KeyCode::PageDown) => {
                self.selected = calendar::next_week(self.selected)
            }
            _ => {}
        }
        Outcome::Continue
    }

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
        frame.render_widget(panel_block(" Periods "), right);
    }

    fn render_days(&mut self, frame: &mut Frame, area: Rect) {
        let block = panel_block(" Days ");
        let inner = block.inner(area);
        frame.render_widget(block, area);
        let [header, list] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(inner);
        frame.render_widget(Paragraph::new(" Date       Worked").style(PANEL), header);

        let height = usize::from(list.height);
        let first = self.scroll_to_selected(height);
        let width = usize::from(list.width);
        let lines: Vec<Line> = list_rows(first)
            .take(height)
            .map(|row| match row {
                ListRow::Day(date) => {
                    let worked = self
                        .timesheet
                        .worked_time_on(date)
                        .map(|worked| worked.format_duration())
                        .unwrap_or_default();
                    let text = format!(" {}  {worked:>6}", calendar::format_dutch(date));
                    let style = if date == self.selected {
                        SELECTED
                    } else {
                        PANEL
                    };
                    Line::from(Span::styled(format!("{text:<width$}"), style))
                }
                ListRow::Separator(Separator::Year) => Line::raw("═".repeat(width)),
                ListRow::Separator(_) => Line::raw("─".repeat(width)),
            })
            .collect();
        frame.render_widget(Paragraph::new(lines).style(PANEL), list);
    }

    /// Adjusts the first visible day so the selected day is visible, and returns it.
    fn scroll_to_selected(&mut self, height: usize) -> Date {
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

fn panel_block(title: &str) -> Block<'_> {
    Block::bordered()
        .border_type(BorderType::Double)
        .title(title)
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
        ("↑/↓", "Previous/next day"),
        ("PgUp/PgDn", "Previous/next week"),
        ("F1", "Help"),
        ("F2, Esc", "Back to Tracking"),
        ("Ctrl+Q", "Quit"),
    ];
    let lines: Vec<Line> = shortcuts
        .iter()
        .map(|(key, action)| Line::raw(format!(" {key:<12}{action}")))
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
