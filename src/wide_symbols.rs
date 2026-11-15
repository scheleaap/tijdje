//! Background fix for wide symbols (ADR-0001).
//!
//! Some terminals (e.g. Ptyxis) advance the cursor only one cell for emoji
//! like 🏖️ and ⚠️ but draw them two cells wide, leaving the second cell's
//! background unpainted. After each frame, we paint that cell with a space in
//! the cell's background and redraw the symbol on top of it.

use std::io::{self, Write};

use ratatui::{
    backend::IntoCrossterm,
    buffer::Buffer,
    crossterm::{
        cursor::MoveTo,
        queue,
        style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    },
    text::Span,
};

pub fn paint_backgrounds(buffer: &Buffer, out: &mut impl Write) -> io::Result<()> {
    let area = buffer.area;
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right().saturating_sub(1) {
            let cell = &buffer[(x, y)];
            if Span::raw(cell.symbol()).width() == 2 {
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
    queue!(out, ResetColor)?;
    out.flush()
}
