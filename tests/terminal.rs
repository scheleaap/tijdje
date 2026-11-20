//! Terminal setup: the check for the Kitty keyboard protocol (ADR-0003).
//!
//! Detecting and enabling the protocol needs a real terminal; it is checked by hand in Ghostty.

use std::io;

use tijdje::terminal;

#[test]
fn a_terminal_with_the_protocol_is_accepted() {
    assert_eq!(terminal::require_keyboard_protocol(Ok(true)), Ok(()));
}

#[test]
fn a_terminal_without_the_protocol_is_refused_with_the_supported_terminals() {
    let message = terminal::require_keyboard_protocol(Ok(false)).unwrap_err();
    assert!(message.contains("Kitty keyboard protocol"), "{message}");
    for name in [
        "Ghostty",
        "kitty",
        "foot",
        "Alacritty",
        "WezTerm with `enable_kitty_keyboard`",
    ] {
        assert!(message.contains(name), "{name} missing from: {message}");
    }
}

#[test]
fn a_failed_query_counts_as_no_protocol() {
    let timeout = io::Error::other("The keyboard enhancement status could not be read");
    assert_eq!(
        terminal::require_keyboard_protocol(Err(timeout)),
        terminal::require_keyboard_protocol(Ok(false))
    );
}
