# 13: Clock port

**What to build:** "Today" stays correct while the application runs past midnight.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Today comes from an injected clock; tests use a fake clock
- [x] The event loop wakes up at least once a minute, so a date change is picked up without a key press
- [x] After the date changes, balances and day warnings use the new today; the selected day stays
- [x] The "open period only today" rule uses the current today

Spec: `.scratch/time-tracking/spec.md`
