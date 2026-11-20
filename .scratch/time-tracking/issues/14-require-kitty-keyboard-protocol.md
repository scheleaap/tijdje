# 14: Require the Kitty keyboard protocol

**What to build:** The application checks the terminal on start, enables disambiguated key codes, and refuses to start in terminals without the Kitty keyboard protocol (ADR-0003).

**Blocked by:** None (can start immediately)

**Status:** done

- [x] On start, before the event loop, the application detects Kitty keyboard protocol support; without it, it prints a message naming supported terminals (Ghostty, kitty, foot, Alacritty, WezTerm with `enable_kitty_keyboard`) and exits with a non-zero code
- [x] Disambiguated key codes are enabled on start and restored on exit and on a panic (before leaving the alternate screen)
- [x] Only key presses are handled
- [x] The README documents the Ghostty setting `keybind = ctrl+enter=unbind` and that Ctrl+PgUp/PgDn and Alt+1..8 come through in a window with a single tab
- [ ] Checked by hand in Ghostty: Ctrl+Enter, Ctrl+Backspace, Ctrl+Del, Shift+Tab and Alt+digits arrive distinctly (moved to ticket 16: these keys only do something there; the start-up check, the refusal in Ptyxis, day navigation and restoring the shell were checked)

Spec: `.scratch/time-tracking/spec.md`
