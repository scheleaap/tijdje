# 27: Work periods need a project

**What to build:** A work period without a project fails the consistency check.

**Blocked by:** none

**Status:** ready-for-agent

- [ ] A work period without a project is a consistency failure, also an open period on the current day (domain built test-first)
- [ ] It shows right away as ⚠️ (period indicator and day warning), blocks leaving a changed day, and makes Ctrl+S/Ctrl+Q ask first, like other consistency failures
- [ ] Unchanged older days without a project can still be browsed
- [ ] Existing tests and sample data that relied on work periods without a project are updated; `sample/timesheet.yaml` keeps at least one inconsistent day only where it shows on purpose

Spec: `.scratch/time-tracking/spec.md`
