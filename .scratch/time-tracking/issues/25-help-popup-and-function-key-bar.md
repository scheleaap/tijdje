# 25: Help popup and function-key bar

**What to build:** Help becomes a grouped popup in Midnight Commander's user-menu colors, and the function-key bar fills the full width.

**Blocked by:** 16, 19, 20, 21, 24

**Status:** ready-for-agent

- [ ] F1 opens the help popup; Esc or F1 closes it; it scrolls if it doesn't fit
- [ ] Groups: General, Day navigation, Field navigation, Field editing, Period definition, Period management, listing every shortcut of the spec's Keyboard section
- [ ] Colors: cyan background, light text, yellow title, single-line frame, black shadow
- [ ] The Help screen is gone; F2 Track stays
- [ ] Ctrl+S and Ctrl+Q get a line each again (ticket 24 merged them into "Save/quit" to fit 24 rows until the help scrolls)
- [ ] The function-key bar's ten slots stretch evenly over the full width

Spec: `.scratch/time-tracking/spec.md`
