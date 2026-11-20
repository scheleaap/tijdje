//! Drawing wide symbols reliably (ADR-0001).
//!
//! Some terminals (e.g. Ptyxis) advance the cursor only one cell for emoji
//! like 🏖️ and ⚠️ but draw them two cells wide, leaving the second cell's
//! background unpainted. After each frame, we paint that cell with a space in
//! the cell's background and redraw the symbol on top of it.
//!
//! ratatui also writes the cell covered by such an emoji when its symbol
//! changed (e.g. a separator's `─` before), right after the emoji and without
//! moving the cursor. In terminals that advance two cells (Ghostty, kitty,
//! foot), that write and the rest of the row land one cell too far right and
//! overwrite the next cell, such as a panel border. We therefore keep ratatui
//! from writing covered cells; the repaint above clears them instead.

use std::io::{self, Write};

use ratatui::{
    CompletedFrame, Frame, Terminal,
    backend::{Backend, IntoCrossterm},
    buffer::{Buffer, CellDiffOption},
    crossterm::{
        cursor::{MoveTo, RestorePosition, SavePosition},
        queue,
        style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    },
    text::Span,
};

/// Draws a frame with `render`, then repaints its wide symbols to `out`.
pub fn draw<'a, B: Backend<Error = io::Error>>(
    terminal: &'a mut Terminal<B>,
    out: &mut impl Write,
    render: impl FnOnce(&mut Frame),
) -> io::Result<CompletedFrame<'a>> {
    let frame = terminal.draw(|frame| {
        render(frame);
        skip_covered_cells(frame.buffer_mut());
    })?;
    paint_backgrounds(frame.buffer, out)?;
    Ok(frame)
}

/// Keeps ratatui from writing the cells covered by wide symbols.
fn skip_covered_cells(buffer: &mut Buffer) {
    let area = buffer.area;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right().saturating_sub(1) {
            if is_wide(buffer[(x, y)].symbol()) {
                buffer[(x + 1, y)].set_diff_option(CellDiffOption::Skip);
            }
        }
    }
}

fn is_wide(symbol: &str) -> bool {
    Span::raw(symbol).width() == 2
}

/// The cursor position is saved first and restored afterwards, so the cursor
/// stays where the frame put it (e.g. in the field being edited).
pub fn paint_backgrounds(buffer: &Buffer, out: &mut impl Write) -> io::Result<()> {
    queue!(out, SavePosition)?;
    let area = buffer.area;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right().saturating_sub(1) {
            let cell = &buffer[(x, y)];
            if is_wide(cell.symbol()) {
                queue!(
                    out,
                    SetBackgroundColor(cell.bg.into_crossterm()),
                    SetForegroundColor(cell.fg.into_crossterm()),
                    MoveTo(x + 1, y),
                    Print(" "),
                    MoveTo(x, y),
                    Print(cell.symbol()),
                )?;
            }
        }
    }
    queue!(out, ResetColor, RestorePosition)?;
    out.flush()
}
