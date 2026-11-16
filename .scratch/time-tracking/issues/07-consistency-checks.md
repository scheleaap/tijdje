# 07: Consistency checks

**What to build:** Days are checked for consistency when leaving them, saving and quitting, and failures are shown clearly.

**Blocked by:** 05, 06

**Status:** done

- [x] Checks (domain built test-first): From not empty; no overlaps; From before To; at most one sick, one vacation and one holiday period per day; open period only on the current day; non-working periods have no project
- [x] Before checking, completely empty periods are removed and the rest sorted by From; the sort is not an undo step
- [x] Leaving a day (Ctrl+↑/↓, or ↑/↓/PgUp/PgDn in the left panel) checks it and stays on the day if it fails
- [x] A failure shows a message inside the right panel and highlights the offending period
- [x] Ctrl+S checks all days; on failure: "<day>: <problem> (Go to day / Save anyway / Cancel)"
- [x] Ctrl+Q checks all days first ("Go to day / Quit anyway / Cancel"), then asks "Save before quitting?" if there are unsaved changes
- [x] Days that fail a check at load time are loaded anyway and marked ⚠️ in the left panel

Spec: `.scratch/time-tracking/spec.md`
