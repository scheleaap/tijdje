# 27: Work periods need a project

**What to build:** A work period without a project fails the consistency check.

**Blocked by:** none

**Status:** done

- [x] A work period without a project is a consistency failure, also an open period on the current day (domain built test-first)
- [x] It shows right away as ⚠️ (period indicator and day warning), blocks leaving a changed day, and makes Ctrl+S/Ctrl+Q ask first, like other consistency failures
- [x] Unchanged older days without a project can still be browsed
- [x] Existing tests that relied on work periods without a project are updated, and every work period in `sample/timesheet.yaml` gets a project

Spec: `.scratch/time-tracking/spec.md`
