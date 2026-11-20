# 15: Panels without focus and new day navigation

**What to build:** Focus is always in the right panel: the left panel only shows the selected day, a field always has focus, a day without periods shows an empty period, and days are changed with Ctrl+arrows and Ctrl+PgUp/PgDn.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The left panel has no keys; the selected day always has the cyan bar; panel titles have no highlight
- [ ] Ctrl+↑/↓ select the previous/next day, Ctrl+PgUp/PgDn the previous/next week; leaving a changed day still runs its consistency check
- [ ] A day without periods shows one empty period; it becomes a real period once something is filled in and is never saved while empty
- [ ] A field always has focus; switching days focuses From of the first period; after deleting the only period, the empty period's From has focus
- [ ] Removed: Alt+←/→, Alt+↑/↓, Alt+PgUp/PgDn and the left panel's keys

Spec: `.scratch/time-tracking/spec.md`
