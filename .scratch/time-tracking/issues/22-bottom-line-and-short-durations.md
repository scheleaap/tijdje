# 22: Bottom line and short durations

**What to build:** The right panel's bottom line shows worked time against actual expected working time and until when to work for 8h, and durations under an hour drop the "0h".

**Blocked by:** 21

**Status:** ready-for-agent

- [ ] `Worked [<worked time>] of [<actual expected working time>] | Work until [<time>] for [8h]`, with the bracketed parts on a cyan background
- [ ] "Work until" = base + (8h − worked time) + missing break time; base = latest To of the from–to periods, or the From of an open last period; missing break time = 1h minus the explicit and implied break time, if that is under 1h; free periods don't count as break (domain built test-first)
- [ ] `[–]` when 8h is already worked or the day has no from–to periods
- [ ] Example: 0900–1200 and 1230–1500 gives `Work until [18:00]`
- [ ] Durations under an hour show as `42m` (padded where columns align); balances `+42m`/`-12m`; zero is `0m`

Spec: `.scratch/time-tracking/spec.md`
