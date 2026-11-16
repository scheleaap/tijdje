# 10: Undo/redo

**What to build:** The user can undo and redo accepted changes in the right panel.

**Blocked by:** 06

**Status:** done

- [x] Ctrl+Z/Ctrl+Y undo/redo; ignored while editing (A)
- [x] One step = one accepted change: field value, paste, checkbox toggle (including its auto-fill), adding or deleting a period
- [x] One history for the whole session, across all days; undoing a change on another day selects that day
- [x] Saving keeps the history

Spec: `.scratch/time-tracking/spec.md`
