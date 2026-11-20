//! What the terminal shows after the draw loop's real output: frames drawn through ratatui's
//! `CrosstermBackend` plus the wide-symbol repaint (ADR-0001), interpreted by a small screen
//! model and compared with the buffer ratatui meant to show.

mod common;

use common::{SAMPLE, SAMPLE_TODAY, app_with};
use ratatui::{
    Terminal, TerminalOptions, Viewport,
    backend::CrosstermBackend,
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::Style,
    text::Span,
};
use std::{cell::RefCell, collections::HashSet, io, rc::Rc};
use tijdje::{app::App, wide_symbols};

/// Output that the test reads back while the backend writes to it.
#[derive(Clone, Default)]
struct SharedOutput(Rc<RefCell<Vec<u8>>>);

impl io::Write for SharedOutput {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// How far the terminal moves the cursor after printing an emoji with a variation selector.
#[derive(Clone, Copy, Debug)]
enum EmojiAdvance {
    /// Two cells, like its width (Ghostty, kitty, foot).
    TwoCells,
    /// One cell, while drawing it two cells wide (VTE terminals such as Ptyxis).
    OneCell,
}

/// A terminal screen that applies the escape sequences crossterm emits.
struct Screen {
    cells: Vec<Vec<String>>,
    width: usize,
    cursor: (usize, usize),
    saved: (usize, usize),
    advance: EmojiAdvance,
}

/// The cell to the right of a two-cell-wide symbol, covered by it.
const COVERED: &str = "";

impl Screen {
    fn new(width: u16, height: u16, advance: EmojiAdvance) -> Self {
        let width = usize::from(width);
        Self {
            cells: vec![vec![" ".to_owned(); width]; usize::from(height)],
            width,
            cursor: (0, 0),
            saved: (0, 0),
            advance,
        }
    }

    fn apply(&mut self, bytes: &[u8]) {
        let text = std::str::from_utf8(bytes).expect("UTF-8 output");
        let mut rest = text;
        while !rest.is_empty() {
            if let Some(after_esc) = rest.strip_prefix('\x1b') {
                rest = self.apply_escape(after_esc);
            } else {
                let end = rest.find('\x1b').unwrap_or(rest.len());
                self.print(&rest[..end]);
                rest = &rest[end..];
            }
        }
    }

    /// Applies one escape sequence and returns the text after it.
    fn apply_escape<'a>(&mut self, seq: &'a str) -> &'a str {
        match seq.chars().next() {
            Some('7') => {
                self.saved = self.cursor;
                &seq[1..]
            }
            Some('8') => {
                self.cursor = self.saved;
                &seq[1..]
            }
            _ => self.apply_csi(seq.strip_prefix('[').expect("CSI sequence")),
        }
    }

    /// Applies one CSI sequence (after `ESC [`) and returns the text after it.
    fn apply_csi<'a>(&mut self, csi: &'a str) -> &'a str {
        let end = csi
            .find(|c: char| c.is_ascii_alphabetic())
            .expect("CSI final byte");
        let (params, final_byte) = (&csi[..end], &csi[end..=end]);
        if final_byte == "H" {
            let mut numbers = params.split(';').map(|n| n.parse::<usize>().unwrap_or(1));
            let row = numbers.next().unwrap_or(1);
            let column = numbers.next().unwrap_or(1);
            self.cursor = (column - 1, row - 1);
        }
        // Colors (m) and cursor visibility (l, h) don't change what's printed where.
        &csi[end + 1..]
    }

    fn print(&mut self, text: &str) {
        for grapheme in Span::raw(text).styled_graphemes(Style::default()) {
            let symbol = grapheme.symbol;
            let width = Span::raw(symbol).width();
            let advance = match self.advance {
                EmojiAdvance::OneCell if symbol.contains('\u{FE0F}') => 1,
                _ => width,
            };
            let (x, y) = self.cursor;
            if x < self.width {
                self.put(x, y, symbol, advance);
            }
            self.cursor.0 = x + advance;
        }
    }

    /// Puts `symbol` at (x, y), covering the next cell too if it advances two cells.
    fn put(&mut self, x: usize, y: usize, symbol: &str, advance: usize) {
        let row = &mut self.cells[y];
        // Overwriting either half of a wide symbol erases the whole symbol.
        if row[x] == COVERED && x > 0 {
            row[x - 1] = " ".to_owned();
        }
        if x + 1 < self.width && row[x + 1] == COVERED {
            row[x + 1] = " ".to_owned();
        }
        row[x] = symbol.to_owned();
        if advance == 2 && x + 1 < self.width {
            if x + 2 < self.width && row[x + 2] == COVERED {
                row[x + 2] = " ".to_owned();
            }
            row[x + 1] = COVERED.to_owned();
        }
    }

    /// The cells that differ from `buffer`. Cells covered by a wide symbol in the buffer are
    /// skipped.
    fn differences(&self, buffer: &Buffer) -> Vec<Difference> {
        let covered: HashSet<(u16, u16)> = wide_symbols::wide_cells(buffer)
            .map(|(x, y)| (x + 1, y))
            .collect();
        let area = buffer.area;
        (area.top()..area.bottom())
            .flat_map(|y| (area.left()..area.right()).map(move |x| (x, y)))
            .filter(|position| !covered.contains(position))
            .map(|(x, y)| Difference {
                x,
                y,
                expected: buffer[(x, y)].symbol().to_owned(),
                shown: self.cells[usize::from(y)][usize::from(x)].clone(),
            })
            .filter(|difference| difference.expected != difference.shown)
            .collect()
    }
}

/// A cell whose symbol on the screen differs from the one ratatui meant to show.
#[derive(Debug, PartialEq)]
struct Difference {
    x: u16,
    y: u16,
    expected: String,
    shown: String,
}

const WIDTH: u16 = 120;
const HEIGHT: u16 = 30;

/// Runs the draw loop of `main` for each step, and returns the screen's differences from the
/// intended buffer after the first step that has any.
fn differences_while(
    app: &mut App,
    advance: EmojiAdvance,
    steps: impl IntoIterator<Item = KeyEvent>,
) -> Vec<Difference> {
    let output = SharedOutput::default();
    let backend = CrosstermBackend::new(output.clone());
    let viewport = Viewport::Fixed(Rect::new(0, 0, WIDTH, HEIGHT));
    let mut terminal = Terminal::with_options(backend, TerminalOptions { viewport }).unwrap();
    let mut screen = Screen::new(WIDTH, HEIGHT, advance);
    let mut draw = |app: &mut App| {
        let frame = wide_symbols::draw(&mut terminal, &mut output.clone(), |frame| {
            app.render(frame)
        });
        let buffer = frame.unwrap().buffer.clone();
        screen.apply(&output.0.take());
        screen.differences(&buffer)
    };
    let mut differences = draw(app);
    let mut steps = steps.into_iter();
    while differences.is_empty()
        && let Some(key) = steps.next()
    {
        app.handle_key(key);
        differences = draw(app);
    }
    differences
}

/// Scrolls the day list up one line so that the row with 🏖️ (6 Nov) lands on the line that
/// showed the week separator below it.
fn scroll_vacation_onto_separator() -> Vec<KeyEvent> {
    [KeyCode::PageUp, KeyCode::PageUp, KeyCode::Up]
        .map(|code| KeyEvent::new(code, KeyModifiers::CONTROL))
        .to_vec()
}

fn assert_scrolling_keeps_the_screen_intact(advance: EmojiAdvance) {
    let mut app = app_with(SAMPLE, SAMPLE_TODAY);
    let differences = differences_while(&mut app, advance, scroll_vacation_onto_separator());
    assert_eq!(differences, vec![]);
}

#[test]
fn scrolling_the_days_keeps_the_panel_border_when_emoji_advance_two_cells() {
    assert_scrolling_keeps_the_screen_intact(EmojiAdvance::TwoCells);
}

#[test]
fn scrolling_the_days_keeps_the_panel_border_when_emoji_advance_one_cell() {
    assert_scrolling_keeps_the_screen_intact(EmojiAdvance::OneCell);
}
