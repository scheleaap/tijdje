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
use std::{cell::RefCell, io, rc::Rc};
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
        if let Some(rest) = seq.strip_prefix('7') {
            self.saved = self.cursor;
            return rest;
        }
        if let Some(rest) = seq.strip_prefix('8') {
            self.cursor = self.saved;
            return rest;
        }
        let csi = seq.strip_prefix('[').expect("CSI sequence");
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
            if x >= self.width {
                continue;
            }
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
            self.cursor.0 = x + advance;
        }
    }

    /// The cells that differ from `buffer`, as (x, y, expected, shown). Cells covered by a wide
    /// symbol in the buffer are skipped.
    fn differences(&self, buffer: &Buffer) -> Vec<(u16, u16, String, String)> {
        let area = buffer.area;
        let mut differences = Vec::new();
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                let covered = x > area.left() && buffer[(x - 1, y)].symbol().len() > 1 && {
                    Span::raw(buffer[(x - 1, y)].symbol()).width() == 2
                };
                if covered {
                    continue;
                }
                let expected = buffer[(x, y)].symbol();
                let shown = &self.cells[usize::from(y)][usize::from(x)];
                if expected != shown {
                    differences.push((x, y, expected.to_owned(), shown.clone()));
                }
            }
        }
        differences
    }
}

const WIDTH: u16 = 120;
const HEIGHT: u16 = 30;

/// Runs the draw loop of `main` for each step, and returns the screen's differences from the
/// intended buffer after the first step that has any.
fn differences_while(
    app: &mut App,
    advance: EmojiAdvance,
    steps: impl IntoIterator<Item = KeyEvent>,
) -> Vec<(u16, u16, String, String)> {
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
    let differences = draw(app);
    if !differences.is_empty() {
        return differences;
    }
    for key in steps {
        app.handle_key(key);
        let differences = draw(app);
        if !differences.is_empty() {
            return differences;
        }
    }
    Vec::new()
}

/// Scrolls the day list up one line so that the row with 🏖️ (6 Nov) lands on the line that
/// showed the week separator below it.
fn scroll_vacation_onto_separator() -> Vec<KeyEvent> {
    [KeyCode::PageUp, KeyCode::PageUp, KeyCode::Up]
        .map(|code| KeyEvent::new(code, KeyModifiers::CONTROL))
        .to_vec()
}

#[test]
fn scrolling_the_days_keeps_the_panel_border_when_emoji_advance_two_cells() {
    let mut app = app_with(SAMPLE, SAMPLE_TODAY);
    let steps = scroll_vacation_onto_separator();
    let differences = differences_while(&mut app, EmojiAdvance::TwoCells, steps);
    assert_eq!(differences, vec![], "(x, y, expected, shown)");
}

#[test]
fn scrolling_the_days_keeps_the_panel_border_when_emoji_advance_one_cell() {
    let mut app = app_with(SAMPLE, SAMPLE_TODAY);
    let steps = scroll_vacation_onto_separator();
    let differences = differences_while(&mut app, EmojiAdvance::OneCell, steps);
    assert_eq!(differences, vec![], "(x, y, expected, shown)");
}
