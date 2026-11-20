# 19: Day note

**What to build:** A day can have a note, edited in a full-width field at the top of the right panel and saved with the day.

**Blocked by:** 16

**Status:** ready-for-agent

- [ ] A full-width input field above the period header; ↑ from the first period goes there; it is the first field in Tab order; default focus stays on From of the first period
- [ ] Typing replaces, Enter edits, Del clears, Ctrl+C/V copy/paste, like other input fields
- [ ] Saved as `notes` on the day (`null` when empty); a day with only a note is written
- [ ] Changes are undo steps

Spec: `.scratch/time-tracking/spec.md`
