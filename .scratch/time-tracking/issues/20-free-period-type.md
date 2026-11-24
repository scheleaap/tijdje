# 20: Free period type

**What to build:** A new non-working period type "free" (🆓) records time taken off from overtime; it neither counts as worked time nor reduces the actual expected working time.

**Blocked by:** 16

**Status:** done

- [x] Saved as `type: free` (domain built test-first)
- [x] A Free checkbox column with header 🆓 between Break and Vacation; Alt+f toggles it; the checkboxes stay mutually exclusive
- [x] Free doesn't count as worked time, isn't an absence, has no absence indicator in the left panel, and may not have a project
- [x] The indicator left of From shows 🆓 for a free period without warnings
- [x] Alt+b/f/s/v/h toggle Break/Free/Sick/Vacation/Holiday and accept the active input first

Spec: `.scratch/time-tracking/spec.md`
