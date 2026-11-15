# 02: Balances and day indicators

**What to build:** The left panel shows day balance, total balance, the absence indicator and the warning indicator for every day, calculated from the periods in the timesheet file.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Actual expected working time = regular expected working time (stored per day, constant 432 minutes if absent) minus the duration of the day's vacation, sick and holiday periods (domain built test-first)
- [ ] Day balance = worked time minus actual expected working time; shown signed (`+0h45m`, `-1h12m`)
- [ ] Total balance = sum of day balances from the first recorded day up to and including that day
- [ ] Empty weekdays after the first recorded day count with their full regular expected working time
- [ ] Days before the first recorded day show no balances and no warnings; future days show neither day balance nor total balance
- [ ] Absence column shows at most one emoji, in priority 🤒 (sick) > 🏖️ (vacation) > 🎄 (holiday)
- [ ] Separate warning column shows ⚠️ for days after the first recorded day and before today that are empty or contain an open period
- [ ] After each frame, the second cell of every wide symbol is painted with a space in its background and the symbol redrawn on top (ADR-0001; snippet in the spec, from the prototype)
- [ ] Emoji columns line up and have a fully blue background in Ptyxis

Spec: `.scratch/time-tracking/spec.md`
