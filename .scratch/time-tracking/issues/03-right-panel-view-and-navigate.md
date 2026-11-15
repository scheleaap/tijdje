# 03: Right panel: view and navigate periods

**What to build:** The right panel shows the periods of the selected day and lets the user move focus between panels, periods and fields.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Columns From, To, Duration, Project, Notes, Break, Vacation, Sick, Holiday; From/To exactly 4 characters wide, shown as HHMM
- [ ] Input fields have a darker blue background instead of borders; checkboxes show as `[x]`/`[ ]`
- [ ] Duration is computed per period (`1h55m`); empty for an open period
- [ ] "→ ☕" to the right of a period that is not directly followed by the next period
- [ ] "⚠️" to the right of a period that starts before 06:00 or ends after 20:00
- [ ] Bottom of the panel shows `Actual expected: 3h36m (regular 7h12m − 3h36m vacation)`, or just `Actual expected: 7h12m` when nothing reduces it
- [ ] Alt+←/→ move focus between the left and right panel
- [ ] Entering a day leaves no period and no field focused
- [ ] ↑/↓ move focus to the same field in the previous/next period; with nothing focused, to the first field of the first/last period
- [ ] Tab/Shift-Tab move through From, To, Project, Notes, Break, Vacation, Sick, Holiday (Duration is skipped)
- [ ] Ctrl+↑/↓ select the previous/next day from the right panel
- [ ] The panel scrolls vertically when periods don't fit

Spec: `.scratch/time-tracking/spec.md`
