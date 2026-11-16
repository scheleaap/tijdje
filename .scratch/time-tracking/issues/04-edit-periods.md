# 04: Edit periods

**What to build:** The user can edit period fields in place, add periods and delete them.

**Blocked by:** 03

**Status:** done

- [x] Typing in a focused field replaces its value; the typed text is active input until accepted (Enter) or rejected (Esc, restoring the previous value)
- [x] ←/→/Home/End move the cursor while editing
- [x] From/To accept exactly four digits HHMM, 0000–2359; invalid input is rejected with a message in the right panel
- [x] Enter (not editing) in the last period, or on a day with periods but none focused, creates a period whose From is the previous To, focused on From
- [x] Enter on a day without periods creates the first period with an empty From
- [x] Ctrl+Del (not editing) asks "Delete period? (Delete/Cancel)" and deletes the focused period
- [x] Popup buttons: ←/→ + Enter, first letter, Esc cancels
- [x] Shortcuts marked (A) in the spec are ignored while editing, including Ctrl+↑/↓ (decided: it does not accept active input)
- [x] Long notes are cut off with `…` and scroll horizontally while editing
- [x] Worked time, balances and indicators update after each accepted change

Spec: `.scratch/time-tracking/spec.md`
