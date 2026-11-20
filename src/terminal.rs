//! Terminal setup: checks for and enables the Kitty keyboard protocol, and restores the terminal
//! on exit and on a panic (ADR-0003).

use std::{
    fmt,
    io::{self, Write},
    sync::atomic::{AtomicBool, Ordering},
};

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

/// Terminals known to support the Kitty keyboard protocol (ADR-0003).
const SUPPORTED_TERMINALS: &str =
    "Ghostty, kitty, foot, Alacritty 0.13+ or WezTerm with `enable_kitty_keyboard`";

/// The terminal doesn't support the Kitty keyboard protocol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnsupportedTerminal;

impl fmt::Display for UnsupportedTerminal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "this terminal doesn't support the Kitty keyboard protocol.\n\
             Use a terminal that does: {SUPPORTED_TERMINALS}."
        )
    }
}

impl std::error::Error for UnsupportedTerminal {}

/// Refuses a terminal without the Kitty keyboard protocol, given the result of asking the
/// terminal for it. A failed query, e.g. a timeout, counts as no protocol.
pub fn require_keyboard_protocol(support: io::Result<bool>) -> Result<(), UnsupportedTerminal> {
    match support {
        Ok(true) => Ok(()),
        Ok(false) | Err(_) => Err(UnsupportedTerminal),
    }
}

/// Asks the terminal whether it supports the Kitty keyboard protocol, waiting up to about two
/// seconds for the answer. Call it before [`init`].
pub fn check_keyboard_protocol() -> Result<(), UnsupportedTerminal> {
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
    KEYBOARD_FLAGS_PUSHED.store(true, Ordering::SeqCst);
    Ok(terminal)
}

/// Restores the key codes, then leaves the alternate screen and raw mode.
pub fn restore() {
    if let Err(e) = pop_keyboard_flags() {
        eprintln!("tijdje: failed to restore the keyboard mode: {e}");
    }
    ratatui::restore();
}

/// Whether disambiguated key codes are enabled, so they're restored exactly
/// once: on exit or on a panic, and only if enabling them worked.
static KEYBOARD_FLAGS_PUSHED: AtomicBool = AtomicBool::new(false);

fn pop_keyboard_flags() -> io::Result<()> {
    if !KEYBOARD_FLAGS_PUSHED.swap(false, Ordering::SeqCst) {
        return Ok(());
    }
    let mut stdout = io::stdout();
    execute!(stdout, PopKeyboardEnhancementFlags)?;
    stdout.flush()
}
