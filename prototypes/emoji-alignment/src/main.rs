//! PROTOTYPE, throwaway. Question (ADR-0001): do 🏖️/⚠️ line up in Ptyxis when
//! every cell is written with explicit cursor positioning?
//! Keys: 1-5 switch variant, q quits.

use std::io::{stdout, Write};

use ratatui::{
    crossterm::{
        cursor::MoveTo,
        event::{self, Event, KeyCode, KeyEventKind},
        queue,
        style::{Color as CColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
        terminal::{Clear, ClearType},
    },
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};
use unicode_width::UnicodeWidthStr;

const VARIANTS: [&str; 5] = [
    "1: ratatui default, 🏖️ ⚠️",
    "2: ratatui + explicit cursor positioning per cell, 🏖️ ⚠️",
    "3: ratatui default, fallback 🌴 ❗",
    "4: raw printing, no positioning (like printf)",
    "5: ratatui default + blue space under each wide emoji, 🏖️ ⚠️",
];

fn rows(fallback: bool) -> Vec<[&'static str; 5]> {
    let (vac, warn) = if fallback { ("🌴", "❗") } else { ("🏖️", "⚠️") };
    vec![
        ["ma  9 nov", "6h43m", "+0h17m", "", ""],
        ["di 10 nov", "6h51m", "+0h25m", vac, ""],
        ["wo 11 nov", "7h52m", "+1h05m", "🤒", warn],
        ["do 12 nov", "4h25m", "-1h42m", "🎄", warn],
        ["vr 13 nov", "     ", "      ", vac, warn],
        ["1235-1430", "1h55m", "otto  ", "☕", warn],
    ]
}

/// Pads an emoji to exactly 2 cells according to unicode-width.
fn cell2(s: &str) -> String {
    format!("{s}{}", " ".repeat(2usize.saturating_sub(s.width())))
}

fn line_text(r: &[&str; 5]) -> String {
    format!("{} │{} │{} │{}│{}│ end", r[0], r[1], r[2], cell2(r[3]), cell2(r[4]))
}

fn widths() -> String {
    ["🏖️", "⚠️", "🤒", "🎄", "☕", "🌴", "❗"]
        .iter()
        .map(|e| format!("{e}={}", e.width()))
        .collect::<Vec<_>>()
        .join("  ")
}

fn draw(f: &mut Frame, variant: usize) {
    let style = Style::new().bg(Color::Blue).fg(Color::White);
    let mut lines: Vec<Line> = rows(variant == 2)
        .iter()
        .map(|r| Line::from(Span::raw(line_text(r))))
        .collect();
    lines.push(Line::raw(""));
    lines.push(Line::raw(format!("unicode-width: {}", widths())));
    lines.push(Line::raw("All │ and 'end' should form straight columns."));
    lines.push(Line::raw("Keys: 1-5 switch variant, q quits"));
    let block = Block::bordered().title(VARIANTS[variant]).style(style);
    let area = Rect::new(0, 0, 70.min(f.area().width), 14.min(f.area().height));
    f.render_widget(Paragraph::new(lines).block(block).style(style), area);
}

fn to_ct(c: Color) -> CColor {
    match c {
        Color::Blue => CColor::DarkBlue,
        Color::White => CColor::White,
        _ => CColor::Reset,
    }
}

/// Variant 2: repaint every cell at an absolute position, skipping the cells
/// covered by a wide symbol so the emoji's right half isn't overwritten.
fn repaint_absolute(buf: &ratatui::buffer::Buffer) -> std::io::Result<()> {
    let mut out = stdout();
    let a = buf.area;
    for y in a.top()..a.bottom() {
        let mut skip = 0;
        for x in a.left()..a.right() {
            if skip > 0 {
                skip -= 1;
                continue;
            }
            let c = &buf[(x, y)];
            let sym = c.symbol();
            skip = sym.width().saturating_sub(1);
            queue!(
                out,
                MoveTo(x, y),
                SetBackgroundColor(to_ct(c.bg)),
                SetForegroundColor(to_ct(c.fg)),
                Print(sym)
            )?;
        }
    }
    queue!(out, ResetColor)?;
    out.flush()
}

/// Variant 5: for every wide symbol, paint its second cell with a space in the
/// cell's background first, then redraw the symbol on top of it.
fn underpaint_wide(buf: &ratatui::buffer::Buffer) -> std::io::Result<()> {
    let mut out = stdout();
    let a = buf.area;
    for y in a.top()..a.bottom() {
        for x in a.left()..a.right() {
            let c = &buf[(x, y)];
            let sym = c.symbol();
            if sym.width() == 2 && x + 1 < a.right() {
                queue!(
                    out,
                    SetBackgroundColor(to_ct(c.bg)),
                    SetForegroundColor(to_ct(c.fg)),
                    MoveTo(x + 1, y),
                    Print(" "),
                    MoveTo(x, y),
                    Print(sym)
                )?;
            }
        }
    }
    queue!(out, ResetColor)?;
    out.flush()
}

/// Variant 4: print lines sequentially, letting the terminal advance the cursor.
fn paint_raw() -> std::io::Result<()> {
    let mut out = stdout();
    queue!(out, Clear(ClearType::All), MoveTo(0, 0))?;
    queue!(out, Print(format!("{}\r\n\r\n", VARIANTS[3])))?;
    for r in rows(false) {
        queue!(out, Print(format!("{}\r\n", line_text(&r))))?;
    }
    queue!(out, Print(format!("\r\nunicode-width: {}\r\nKeys: 1-5 switch variant, q quits", widths())))?;
    out.flush()
}

fn main() -> std::io::Result<()> {
    let mut terminal = ratatui::init();
    let mut variant = 0;
    let result = (|| loop {
        if variant == 3 {
            paint_raw()?;
        } else {
            let frame = terminal.draw(|f| draw(f, variant))?;
            let buf = frame.buffer.clone();
            if variant == 1 {
                repaint_absolute(&buf)?;
            } else if variant == 4 {
                underpaint_wide(&buf)?;
            }
        }
        if let Event::Key(k) = event::read()? {
            if k.kind != KeyEventKind::Press {
                continue;
            }
            match k.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char(c @ '1'..='5') => {
                    variant = c as usize - '1' as usize;
                    terminal.clear()?;
                }
                _ => {}
            }
        }
    })();
    ratatui::restore();
    result
}
