# 21: Part of day

**What to build:** A period can have a part of day (full day, 1st half, 2nd half) instead of From and To.

**Blocked by:** 16, 20

**Status:** ready-for-agent

- [ ] Saved as `part_of_day: full_day | first_half | second_half` with `from`/`to` as `null` (domain built test-first)
- [ ] Duration: 100% or 50% of the regular expected working time; counts like any other period in all calculations
- [ ] On screen, "Full day", "1st half" or "2nd half" replaces From and To as one field; Enter and typing do nothing on it; Del switches back to from–to
- [ ] Alt+0 switches to from–to mode with empty times; Alt+1/2/3 switch to full day / 1st half / 2nd half; they accept the active input first
- [ ] Allowed for work, free, vacation, sick and holiday; a part of day on a break fails the consistency check
- [ ] Checks: a period needs a From or a part of day; at most one full day, at most one of each half, no full day together with a half
- [ ] Sort order: full day, 1st half, from–to periods by From, 2nd half; overlaps and implied breaks only between from–to periods
- [ ] Checking vacation, sick or holiday on the day's only empty period sets it to full day; unchecking undoes that while it's still the only period and a full day

Spec: `.scratch/time-tracking/spec.md`
