# 16: Field navigation and editing keys

**What to build:** The new keys for moving between fields, editing them and managing periods work as specified in the spec's Keyboard section.

**Blocked by:** 14, 15

**Status:** ready-for-agent

- [ ] ←/→ move to the field left/right (stopping at the row's edges); ↑/↓ to the closest field in the row above/below, skipping implied-break rows
- [ ] Tab/Shift-Tab walk through all fields in order, wrapping around within the day: per period From, To, Project, Break, Vacation, Sick, Holiday, Notes (tickets 19 and 20 add the day note and Free)
- [ ] Ctrl+Enter moves to the next period's From; in the last period it adds a period after it (not while the last period is open), focused on its From, which is the previous period's To
- [ ] Enter toggles a checkbox, starts editing an input field with its existing text, or accepts the active input; Esc rejects
- [ ] Typing replaces an input field's value, Notes included
- [ ] Del (not editing) clears an input field or unchecks a checkbox
- [ ] While editing: ←/→ move the cursor; Home/Ctrl+A, End/Ctrl+E; Backspace/Del; Ctrl+Backspace/Ctrl+W delete the previous word; Ctrl+U to the start; Ctrl+K to the end; Ctrl+←/→ by word (a word is anything between spaces)
- [ ] Ctrl+Del deletes the focused period after confirming; not while editing; nothing on the placeholder empty period
- [ ] While editing, Tab/Shift-Tab and Ctrl+Enter accept and move on (staying put if accepting fails); day navigation, Ctrl+Del, ↑/↓, Ctrl+S, Ctrl+Q, F1 and Ctrl+Z/Y are silently ignored
- [ ] Removed: Alt+Enter, Alt+Del, Alt+p/n
- [ ] Clearing a field and Ctrl+Enter's added period are undo steps

Spec: `.scratch/time-tracking/spec.md`
