//! Terminal setup: checks for and enables the Kitty keyboard protocol, and restores the terminal
//! on exit and on a panic (ADR-0003).

use std::io::{self, Write};

use ratatui::{
    DefaultTerminal,
    crossterm::{
        event::{
            KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
        },
        execute,
        terminal::supports_keyboard_enhancement,
    },
};

/// Refuses a terminal without the Kitty keyboard protocol, given the result of asking the
/// terminal for it. A failed query, e.g. a timeout, counts as no protocol.
pub fn require_keyboard_protocol(support: io::Result<bool>) -> Result<(), String> {
    match support {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(
            "this terminal doesn't support the Kitty keyboard protocol.\n\
             Use a terminal that does: Ghostty, kitty, foot, Alacritty 0.13+ \
             or WezTerm with `enable_kitty_keyboard`."
                .to_owned(),
        ),
    }
}

/// Asks the terminal whether it supports the Kitty keyboard protocol, waiting up to about two
/// seconds for the answer. Call it before [`init`].
pub fn check_keyboard_protocol() -> Result<(), String> {
    require_keyboard_protocol(supports_keyboard_enhancement())
}

/// Enters the alternate screen with raw mode and enables disambiguated key codes. [`restore`]
/// undoes it; a panic undoes it too.
pub fn init() -> io::Result<DefaultTerminal> {
    // ratatui installs a panic hook that leaves the alternate screen. The hook installed after
    // it runs first, so the key codes are restored on the alternate screen, whose flag stack is
    // separate from the main screen's.
    let terminal = ratatui::init();
    let leave_alternate_screen = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = pop_keyboard_flags();
        leave_alternate_screen(info);
    }));
    if let Err(e) = execute!(
        io::stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    ) {
        restore();
        return Err(e);
    }
    Ok(terminal)
}

/// Restores the key codes, then leaves the alternate screen and raw mode.
pub fn restore() {
    if let Err(e) = pop_keyboard_flags() {
        eprintln!("Failed to restore the keyboard mode: {e}");
    }
    ratatui::restore();
}

fn pop_keyboard_flags() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, PopKeyboardEnhancementFlags)?;
    stdout.flush()
}
