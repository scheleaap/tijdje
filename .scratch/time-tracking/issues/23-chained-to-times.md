# 23: Chained To times

**What to build:** Changing a period's To also changes the next period's From when that was equal to the old To.

**Blocked by:** 16

**Status:** done

- [x] Only To → next from–to period's From; changing a From changes nothing else (domain built test-first)
- [x] The chained change is part of the same undo step

Spec: `.scratch/time-tracking/spec.md`
