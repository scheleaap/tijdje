# 06: Period types

**What to build:** Periods can be marked as break, vacation, sick or holiday, which affects worked time, actual expected working time and the indicators.

**Blocked by:** 04

**Status:** done

- [x] Break/Vacation/Sick/Holiday checkboxes are mutually exclusive; none checked means work
- [x] Space toggles the focused checkbox; Alt+b/v/s/h toggle Break/Vacation/Sick/Holiday from anywhere in the period, accepting active input first
- [x] Checking Vacation, Sick or Holiday on a day whose only period has empty From and To fills From 0900 and To 0900 + regular expected working time (e.g. 1612); unchecking empties From and To again
- [x] Breaks don't count toward worked time; vacation, sick and holiday periods reduce the actual expected working time
- [x] Balances, absence emoji and the actual expected working time line update immediately
- [x] Alt+p and Alt+n move focus to Project and Notes, accepting active input first

Spec: `.scratch/time-tracking/spec.md`
