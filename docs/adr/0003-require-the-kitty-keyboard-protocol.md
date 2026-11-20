# Require the Kitty keyboard protocol

The keyboard scheme uses combinations that legacy terminal encoding can't tell apart, such as Ctrl+Enter (sent as plain Enter), Ctrl+Backspace (sent as Ctrl+H) and Alt+digits. The application therefore requires a terminal with the Kitty keyboard protocol (Ghostty, kitty, foot, Alacritty 0.13+, WezTerm with `enable_kitty_keyboard`), enables disambiguated key codes on start, and refuses to start with a clear message when the terminal doesn't support it. This replaces the terminal list in ADR-0001; its wide-symbol background fix stays.

## Considered Options

- Keys that work in legacy terminals (Alt+Enter, Alt+Del, …), as used until the second design round: works in VTE terminals such as Ptyxis and in Konsole, but limits the scheme and needs workarounds per key.

## Consequences

- VTE terminals (GNOME Terminal, Ptyxis) and Konsole are not supported until they implement the protocol (VTE issue 2601, Konsole bug 435975).
- In Ghostty, Ctrl+Enter is bound to full screen by default; the README documents `keybind = ctrl+enter=unbind`. Ctrl+PgUp/PgDn and Alt+1..8 switch tabs, but come through in a window with a single tab.
