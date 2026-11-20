# Time tracking

Recording the working time of a consultant, per weekday, assigned to projects.

## Language

**Day**:
A weekday (Monday–Friday) for which periods can be recorded.

**Period**:
A span of a day, from a start time to an end time or as a part of day, optionally assigned to a project.
_Avoid_: Entry, block, slot

**Part of day**:
A period's extent given as a share of the regular expected working time instead of times: full day, 1st half or 2nd half.

**Day note**:
A remark attached to a day as a whole.

**Project**:
The work a period is attributed to, e.g. a customer engagement or internal work.

**Empty period**:
A period without any values: no start or end time, project or notes, and not marked as a non-working period.

**Open period**:
A period whose end time is not yet known, because it is still running.

**Work period**:
A period during which the consultant works on a project.

**Non-working period**:
A period that is a break, free period, vacation period, sick period or holiday period.

**Free period**:
A non-working period in which the consultant works less on purpose, using overtime; it is not an absence.

**Break**:
A non-working period that is not worked and does not reduce the actual expected working time.

**Implied break**:
A gap between two consecutive from–to periods on the same day.

**Vacation period**:
A period during which the consultant is on vacation.

**Sick period**:
A period during which the consultant is sick.

**Holiday period**:
A period that falls on a public holiday.

**Unusual time**:
A period that starts before 06:00 or ends after 20:00.

**Absence**:
A vacation, sick or holiday period: a non-working period that reduces the actual expected working time.

**Regular expected working time**:
The time the consultant is expected to work on a day without vacation, sick or holiday periods.
_Avoid_: Expected working time (without "regular" or "actual")

**Actual expected working time**:
A day's regular expected working time minus the duration of its vacation, sick and holiday periods.
_Avoid_: Expected working time (without "regular" or "actual")

**Worked time**:
The total duration of a day's work periods.
_Avoid_: Worked hours

**Day balance**:
A day's worked time minus its actual expected working time.

**Total balance**:
The sum of all day balances from the first recorded day up to and including a given day.

**Consistency check**:
The rules a day's periods must follow, e.g. no overlaps and From before To.

**Period warning**:
The ❗ indicator for a period that is allowed but likely a mistake: an unusual time, a duration of 0m, or a needless split from the previous period.

**Consistency failure**:
A violation of the consistency check, shown as ⚠️.

**Day warning**:
The ⚠️ indicator in the day list for a day with a consistency failure or an empty past weekday.

**Field**:
Anything in the right panel that can have focus: an input field or a checkbox.

**Input field**:
A field you type a value into, e.g. From, Project or the day note.

**Active input**:
Text typed into an input field that has not yet been accepted or rejected.

**Global notification**:
A short message centered over the screen, shown until the next key press.
