# tijdje

A keyboard-driven terminal application for tracking working time, in the style of Norton Commander. Written in Rust.

## Terminal

tijdje needs a terminal with the [Kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/), such as Ghostty, kitty, foot, Alacritty 0.13+ or WezTerm with `enable_kitty_keyboard`. In other terminals it refuses to start.

In Ghostty:

- Ctrl+Enter toggles full screen by default. Add this to Ghostty's configuration to let it through:

  ```
  keybind = ctrl+enter=unbind
  ```

- Ctrl+PgUp/PgDn switch tabs, but come through in a window with a single tab.
- Alt+1..8 switch tabs. tijdje uses Alt+1..3 for part of day. Add this to Ghostty's configuration to let them through (each key is bound both as a character and as a physical key):

  ```
  keybind = alt+1=unbind
  keybind = alt+digit_1=unbind
  keybind = alt+2=unbind
  keybind = alt+digit_2=unbind
  keybind = alt+3=unbind
  keybind = alt+digit_3=unbind
  ```

## License

GPL-3.0-or-later. See [LICENSE](LICENSE).
