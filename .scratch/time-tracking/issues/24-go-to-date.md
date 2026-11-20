# 24: Go to date

**What to build:** Ctrl+G opens a popup to jump to a typed date or to today.

**Blocked by:** 16, 17

**Status:** ready-for-agent

- [ ] A popup with an input field and the buttons Go, Today and Cancel
- [ ] Accepts `YYYY-MM-DD`, `DD MMM` and `DD MMM YYYY` with Dutch month abbreviations only (`jan feb mrt apr mei jun jul aug sep okt nov dec`); without a year, the current year (domain built test-first)
- [ ] A weekend date selects the Friday before; an invalid date shows a global notification
- [ ] Going to a date leaves the current day like other day navigation, including its consistency check
- [ ] Ignored while editing

Spec: `.scratch/time-tracking/spec.md`
