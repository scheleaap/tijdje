# 18: Warnings and indicators

**What to build:** Period warnings (❗) and consistency failures (⚠️) are told apart, a needlessly split period gets a warning, and the indicator left of From shows the period type of non-working periods.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Period warning ❗: unusual time, duration of 0m, or a from–to period that directly follows the previous from–to period in time (its From equals that To) with the same project, period type and notes, also when both have none (on the second one) (domain built test-first)
- [ ] The indicator left of From shows ⚠️ for periods involved in the day's current consistency problem (determined live on a sorted copy without empty periods, mapped back to the periods on screen), else ❗, else the non-working type's emoji (☕ 🏖️ 🤒 🎄; 🆓 once ticket 20 is in), else nothing
- [ ] The left panel's day warning ⚠️ shows for days that fail the consistency check and for empty past weekdays after the first recorded day (a day with only a day note counts as empty and isn't a recorded day); ❗ never appears in the left panel
- [ ] An empty past day is not a consistency failure: saving and quitting don't ask about it

Spec: `.scratch/time-tracking/spec.md`
