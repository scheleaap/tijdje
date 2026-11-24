# 33: Ctrl+Enter moves down, Ctrl+Shift+Enter up

**What to build:** Ctrl+Enter moves down like ↓ and only adds a period in the last period; Ctrl+Shift+Enter moves up like ↑; PgDn stops at the last period.

**Blocked by:** 32

**Status:** ready-for-agent

- [ ] Ctrl+Enter in a period that isn't the last moves down like ↓, keeping the column; in the last period it adds a period focused on its From, or shows "The last period is still open"; on the day note it does nothing
- [ ] Ctrl+Shift+Enter moves up like ↑, keeping the column; on the first period it does nothing; from the day note it goes to From of the last period
- [ ] While editing, both accept first (taking a visible project suggestion), then move; if accepting fails, focus stays
- [ ] PgDn stops at the last period; only ↓ goes on to the day note
- [ ] The Help window and the README's Ghostty section (`keybind = ctrl+shift+enter=unbind`) are updated

Spec: `.scratch/time-tracking/spec.md`
