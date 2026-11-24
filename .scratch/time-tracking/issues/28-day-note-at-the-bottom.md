# 28: Day note at the bottom

**What to build:** The day note moves to the bottom of the right panel, gets a header and is last in Tab order.

**Blocked by:** none

**Status:** done

- [x] Below the period rows: a header row "Day note" styled like the period column headers, the day note's full-width input field on the line below, then an empty line, then the bottom line
- [x] The header and the input field have the same left padding as the bottom line; the dimmed "Day note" placeholder is gone
- [x] The day note is the last field in Tab order (Tab wraps from it to From of the first period); ↓ from the last period goes to it; ↑ from it goes to From of the last period
- [x] On the day note, ↓, Ctrl+Enter, ←/→, the period-definition keys and Ctrl+Del do nothing

Spec: `.scratch/time-tracking/spec.md`
