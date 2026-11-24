//! Throwaway prototype: shows the key events tijdje receives, with the same
//! keyboard protocol settings as the app. Press Esc twice in a row to quit.
//!
//! Run with `cargo run` in this folder.

use std::io::{self, Write};

use ratatui::crossterm::{
    event::{
        self, Event, KeyCode, KeyEventKind, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, supports_keyboard_enhancement},
};

fn main() -> io::Result<()> {
    let mut out = io::stdout();
    let kitty = supports_keyboard_enhancement()?;
    enable_raw_mode()?;
    if kitty {
        execute!(
            out,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;
    }
    write!(
        out,
        "Kitty keyboard protocol: {}\r\nPress keys; Esc twice quits.\r\n",
        if kitty { "yes" } else { "NO" }
    )?;
    out.flush()?;

    let mut previous_was_esc = false;
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            write!(out, "{:?} + {:?}\r\n", key.code, key.modifiers)?;
            out.flush()?;
            let is_esc = key.code == KeyCode::Esc;
            if is_esc && previous_was_esc {
                break;
            }
            previous_was_esc = is_esc;
        }
    }

    if kitty {
        execute!(out, PopKeyboardEnhancementFlags)?;
    }
    disable_raw_mode()
}
