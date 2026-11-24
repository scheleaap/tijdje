# 21: Part of day

**What to build:** A period can have a part of day (full day, 1st half, 2nd half) instead of From and To.

**Blocked by:** 16, 20

**Status:** done

- [x] Saved as `part_of_day: full_day | first_half | second_half` with `from`/`to` as `null` (domain built test-first)
- [x] Duration: full day 100% of the regular expected working time; 1st half is half of it rounded down to the minute, 2nd half the rest; counts like any other period in all calculations; a part-of-day period is never open
- [x] On screen, "Full day", "1st half" or "2nd half" replaces From and To as one 9-cell field; Duration shows its duration; Enter and typing do nothing on it; Del switches back to from–to (focus to From); for ↑/↓ it counts as standing above/below both From and To, and leaving it towards a from–to row lands on From; Ctrl+C/V do nothing on it
- [x] Alt+0 switches a part-of-day period to from–to mode with empty times (focus to From; on a from–to period it does nothing); Alt+1/2/3 switch to full day / 1st half / 2nd half (focus to the part-of-day field); they accept the active input first; on the day note, Alt+0..3 do nothing
- [x] Allowed for work, free, vacation, sick and holiday; a part of day on a break fails the consistency check
- [x] Checks, after the existing ones: a period needs a From or a part of day; `A break can't be a part of day (Full day)`; `More than one full day`; `More than one 1st half`/`2nd half`; `Full day together with a half`; part-of-day periods are named by their label in messages
- [x] Sort order: full day, 1st half, from–to periods by From, 2nd half; overlaps and implied breaks only between from–to periods
- [x] Checking vacation, sick or holiday on the day's only period without times, part of day and project (notes may be set) sets it to full day (times are never auto-filled anymore; unchecking never clears times); unchecking (Enter, Alt or Del) removes the full day while it's still the only period and a full day, keeping the notes
- [x] `part_of_day` with a non-null `from`/`to` is a parse error
- [x] Ctrl+Enter after a part-of-day last period adds a period with an empty From
- [x] The empty-period definition excludes periods with a part of day (they are never removed when sorting)
- [x] Chained times (ticket 23, `Day::set_to`) skip part-of-day and empty periods when looking for the next from–to period

Spec: `.scratch/time-tracking/spec.md`
