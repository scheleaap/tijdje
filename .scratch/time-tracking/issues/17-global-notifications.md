# 17: Global notifications

**What to build:** All messages appear as one-line global notifications, centered with a cyan background over the row just above the panels' bottom border, until the next key press; saving shows "Saved".

**Blocked by:** 15

**Status:** ready-for-agent

- [ ] Invalid time, failed consistency check, clipboard messages and "Nothing to undo/redo" are global notifications
- [ ] The right panel's message row is gone
- [ ] Ctrl+S shows "Saved" after a successful save; not when saving while quitting
- [ ] A notification floats over whatever is on that row and disappears at the next key press

Spec: `.scratch/time-tracking/spec.md`
