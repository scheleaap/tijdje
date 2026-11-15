# 08: Clipboard

**What to build:** The user can copy and paste field values via the system clipboard.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] Ctrl+C copies the focused field's value to the system clipboard (arboard)
- [ ] Ctrl+V puts the clipboard text into the focused field as accepted input, replacing any active input
- [ ] Pasting `12:35` into From/To gives `1235`; invalid times are rejected with a message in the right panel
- [ ] Ctrl+V on a checkbox does nothing
- [ ] Tests use a fake clipboard

Spec: `.scratch/time-tracking/spec.md`
