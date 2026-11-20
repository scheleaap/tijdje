# 14: Require the Kitty keyboard protocol

**What to build:** The application checks the terminal on start, enables disambiguated key codes, and refuses to start in terminals without the Kitty keyboard protocol (ADR-0003).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] On start, before the event loop, the application detects Kitty keyboard protocol support; without it, it prints a message naming supported terminals (Ghostty, kitty, foot, Alacritty, WezTerm with `enable_kitty_keyboard`) and exits with a non-zero code
- [ ] Disambiguated key codes are enabled on start and restored on exit and on a panic (before leaving the alternate screen)
- [ ] Only key presses are handled
- [ ] The README documents the Ghostty setting `keybind = ctrl+enter=unbind` and that Ctrl+PgUp/PgDn and Alt+1..8 come through in a window with a single tab
- [ ] Checked by hand in Ghostty: Ctrl+Enter, Ctrl+Backspace, Ctrl+Del, Shift+Tab and Alt+digits arrive distinctly

Spec: `.scratch/time-tracking/spec.md`
