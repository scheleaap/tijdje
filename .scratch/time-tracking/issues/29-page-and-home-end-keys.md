# 29: PgUp/PgDn and Home/End

**What to build:** PgUp/PgDn move 10 periods; Home/End move to the first/last field of the period.

**Blocked by:** 28

**Status:** ready-for-agent

- [ ] PgUp/PgDn (not editing) move focus 10 periods up/down, keeping the field, stopping at the first period and at the day note (the row after the last period); from the day note, PgUp goes to From of the period 10 above it, or of the first period
- [ ] Home/End (not editing) move to From/Notes of the focused period; on the day note they do nothing
- [ ] While editing, Home/End keep moving the cursor and PgUp/PgDn are ignored
- [ ] The Help screen lists the new keys

Spec: `.scratch/time-tracking/spec.md`

## Comments

- From ticket 28: Home/End must do nothing on the day note. Outside editing they are not handled yet, so keep the day note out of the new Home/End handling.
