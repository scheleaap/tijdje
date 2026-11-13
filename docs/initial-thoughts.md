I would like to build an application to track my working time. I want to focus on time tracking first. In a later milestone, I want to add syncing with / propagation to Moco. I work at a consulting company.

I want to record time periods and assign them to projects.

The application must be implemented in Rust.

The application should be a TUI (text-based user interface) in the style of Norton Commander, but also modern like Claude Code.

The look and feel should be based on Norton Commander.

The application has "screens" that one can switch between using the function keys.
The bottommost line is a row that shows the function key functions.

The initial screen we will develop is the "tracking" screen.

The tracking screen has two "panels", left and right.
The focus lies either in the left or the right panel.

The left panel shows a vertical list with these columns: date | worked hours | day balance | total balance. Exactly 1 day is always selected in the left panel.
On startup, the current day is selected by default.
Only Monday-Friday are shown. A separator is shown between Friday and the following Monday.

The right panel is the place where "periods" are recorded for the selected date.
The right panel has 0 or more periods. It kind of looks like this:

   From   To    Duration  Project        Notes                    Break Vacation Sick Holiday
  |1235| |1430| 1h55m    |otto        | |                       | [x]   [x]      [x]  [x]

Above, I have marked input fields with `|...|`. The input field width is just to illustrate the relative width. Only From and To should be exactly 4 characters wide. I intentionally want to enter times without a colon.
I have marked checkboxes with `[x]`.

You can just start typing in a field, but must accept/reject the "active input" (i.e. the input that has been typed but not accepted/rejected yet)

In the right panel, the focus can be in 0 or 1 period. By default, no period nor field has focus.

(A) = not possible when editing a field
(B) = a consistency check of the day is performed; cancels the action if the check fails
(C) = accepts active input

Right panel shortcuts:
* Ctrl+↑/↓ selects to the previous/next day. (A) (B) (C)
* Tab/Shift-Tab moves focus to another field in a period, from left to right or right to left.
* ↑/↓ move focus to the same field in the previous/next period. If no period has focus yet, it moves focus to the first field in the first/last period. (A)
* Enter:
  * When editing a field, accepts the active input
  * When not editing field, if the focus is in the last period of the selected day, creates a new period and moves focus to the first field of the new period. The To time of the last period becomes the From time of the new period.
* Esc: When editing a field, rejects the active input and reverts to the previously accepted value
* Alt+b checks/unchecks the break checkbox (C)
* Alt+v checks/unchecks the vacation checkbox (C). 
* Alt+s checks/unchecks the sick checkbox (C).
* Alt+p moves focus to the project field (C)
* Alt+n moves focus to the notes field (C)
* Ctrl+C: Copy the value of the current field to the system clipboard
* Ctrl+V: Paste the value from the system clipboard

If the vacation checkbox is checked: If there is exactly 1 period and From and To are empty, From is filled with 0900 and To is filled with <From + expected working time> (e.g. 1612 if the expected working time is 7,2 hours). If it's unchecked again, From and To are emptied.
If the sick checkbox is checked: Same behavior as the vacation checkbox.
If the holiday checkbox is checked: Same behavior as the vacation checkbox.

Left panel shortcuts:
* ↑/↓ move to the previous/next day
* PgUp/PgDn move to the previous/next week

Tracking screen shortcuts:
* Alt+←/→ switches focus between the left and right panels

Global shortcuts:
* Ctrl+Q quits the application (A) (B)
* Ctrl+S saves the current data (A) (B)

At the bottom of the right panel, the expected daily working time is shown.

Consistency checks (not exhaustive):
* There is at most 1 sickness period per day
* There is at most 1 vacation period per day
* There is at most 1 holiday period per day
* There are no overlapping periods
* The period's From is before To
* The periods are organized from early -> late (i.e. no arbitrary order)
* (Allowed: Non-consecutive periods)
* At most one non-working checkbox can be checked
* Non-working periods (break/sick/vacation/holiday) cannot have a project

If two periods are non-consecutive, a little "→ ☕" indicator is shown at the right of the period to highlight that there is an implied break.
If a period starts before 06:00 or ends after 20:00, a "⚠️" indicator is shown at the right of the period.
If 2 or more non-working checkboxes are checked, a "⚠️" indicator is shown at the right of the period.

Total balance adds up from the first recorded day.
Break periods do not count toward worked hours.
Sick/vacation/holiday periods reduce that day's expected working time by their duration.

UI aspects:
* Input fields do not have line borders but have a differently-colored background than the application background.

We will store the expected daily working time as a constant for now: 7,2h = 432 minutes.

The data is stored in a YAML file:
```yaml
days:
  "<date as YYYY-MM-DD>":
    expected_working_time_minutes: 432
    periods:
      - from: "12:35"
        to: "14:30"
        project: "otto"
        notes: foo! bar?
        type: work # or break, holiday, sick, vacation
      # ...
  # ...
```
