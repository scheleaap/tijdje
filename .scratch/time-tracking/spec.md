# Time tracking (milestone 1)

Status: ready-for-agent

## Problem Statement

As a consultant, I record my working time by hand in a free-form text file. Adding up worked time per day, keeping track of how far I am ahead or behind my actual expected working time, and spotting gaps or mistakes (overlaps, forgotten days, open periods) is manual, slow and error-prone. Later, the same data must be booked in Moco, which today means retyping it.

## Solution

A keyboard-driven terminal application (TUI) in Rust, with the look and feel of Norton Commander and the polish of a modern TUI. Its first screen, **Tracking**, has two panels:

- **Left panel**: an endless list of days (Monday–Friday) with worked time, day balance, total balance and indicators.
- **Right panel**: the periods of the selected day, edited in place, with consistency checks and indicators.

Data is stored in a human-readable YAML file. Moco synchronization follows in a later milestone.

## User Stories

### Navigation and layout

1. As a consultant, I want the application to open on the Tracking screen with today selected, so that I can start recording immediately.
2. As a consultant, I want the previous Friday to be selected when I start the application on a weekend, so that I land on the most recent working day.
3. As a consultant, I want to switch screens with function keys, so that navigation feels like Norton Commander.
4. As a consultant, I want the bottom line to show the function-key functions, so that I can discover them without a manual.
5. As a consultant, I want F1 to show a help screen listing all shortcuts, and Esc or F2 to return to Tracking, so that I can look up keys quickly.
6. As a consultant, I want Alt+←/→ to move focus between the left and right panel, so that I can navigate without a mouse.
7. As a consultant, I want a "terminal too small" message when my terminal is smaller than 120×24, so that the layout never breaks silently.

### Left panel (days)

8. As a consultant, I want to see a vertical list of days with the columns date, worked time, day balance, total balance, absence indicator and warning indicator, so that I see my status at a glance.
9. As a consultant, I want exactly one day to be selected at all times, so that the right panel always has a clear context.
10. As a consultant, I want only Monday–Friday to be listed, so that weekends do not clutter the list.
11. As a consultant, I want a single-line separator between every Friday and the following Monday, so that weeks are visually distinct.
12. As a consultant, I want a double-line separator between two days in different years (also when the switch falls between Friday and Monday), so that year boundaries stand out.
13. As a consultant, I want the list to scroll endlessly into the past and the future, so that I can plan vacation and review history.
14. As a consultant, I want ↑/↓ to select the previous/next day and PgUp/PgDn the previous/next week, so that I can move quickly.
15. As a consultant, I want dates in the Dutch format "ma 12 nov", so that they match how I think about my week.
16. As a consultant, I want worked time shown like `1h55m` and balances signed like `+0h45m` and `-1h12m`, so that values are unambiguous.
17. As a consultant, I want the total balance to add up from the first recorded day, so that I know my overall surplus or deficit.
18. As a consultant, I want an empty weekday after the first recorded day to count with its full regular expected working time, so that forgotten days lower my balance until I record them.
19. As a consultant, I want days before the first recorded day to show no balances and no warnings, so that history before I started tracking is not noise.
20. As a consultant, I want future days to show neither day balance nor total balance, so that planned absences do not distort my current balance.
21. As a consultant, I want an absence indicator per day (🤒 sick, 🏖️ vacation, 🎄 holiday; at most one, in that priority), so that I see absences at a glance.
22. As a consultant, I want a ⚠️ in a separate warning column for days before today (and after the first recorded day) that are empty or contain an open period, so that I notice forgotten or unfinished days.
23. As a consultant, I want a ⚠️ for days that failed a consistency check when the file was loaded, so that I can find and fix them.

### Right panel (periods)

24. As a consultant, I want to see the periods of the selected day with From, To, Duration, Project, Notes and the checkboxes Break, Vacation, Sick and Holiday, so that I can record what I did.
25. As a consultant, I want From and To to be exactly four characters wide and entered as HHMM without a colon, so that typing times is fast.
26. As a consultant, I want input fields to have a different background color instead of borders, so that the panel looks clean.
27. As a consultant, I want the duration of each period computed automatically, so that I do not have to calculate it.
28. As a consultant, I want no period and no field to have focus when I enter a day, so that I do not accidentally edit something.
29. As a consultant, I want ↑/↓ to move focus to the same field in the previous/next period, or, if none has focus, to the first field of the first period (↓) or the last period (↑), so that I can move vertically.
30. As a consultant, I want Tab/Shift-Tab to move focus to the next/previous field of a period (including checkboxes), continuing in the next/previous period and stopping at the very first and last field, so that I can move horizontally.
31. As a consultant, I want Alt+↑/↓ and Alt+PgUp/PgDn to select the previous/next day and week from the right panel, so that I can record several days in a row.
32. As a consultant, I want typing in a focused From, To or Project field to replace its value, as in a spreadsheet, and typing in Notes to add to its text, so that overwriting is fast and notes can be extended. Enter on a focused field starts editing with its existing text.
33. As a consultant, I want to accept active input with Enter and reject it with Esc (restoring the previously accepted value), so that I stay in control of every change.
34. As a consultant, I want ←/→/Home/End to move the cursor while editing, so that I can correct typos.
35. As a consultant, I want Alt+Enter to add a new period after the last one, whose From is the previous period's To, with focus on its From field, so that consecutive periods are quick to enter.
36. As a consultant, I want Alt+Enter on a day without periods to create its first period with an empty From, so that I can start a day.
37. As a consultant, I want an open period (empty To) to be allowed on the current day only, so that I can record a period that is still running.
38. As a consultant, I want an open period not to count toward worked time until To is filled in, so that balances only reflect finished work.
39. As a consultant, I want Enter to toggle the focused checkbox, so that checkboxes work like other fields.
40. As a consultant, I want Alt+b/v/s/h to toggle Break/Vacation/Sick/Holiday, accepting any active input first, so that I can set the period type from anywhere in the period.
41. As a consultant, I want the period-type checkboxes to be mutually exclusive, so that a period always has a single type.
42. As a consultant, I want checking Vacation, Sick or Holiday on a day whose only period has empty From and To to fill in From 0900 and To 0900 + regular expected working time (e.g. 1612), and unchecking it to empty From and To again, so that whole-day absences take one keystroke.
43. As a consultant, I want Alt+p and Alt+n to jump to the Project and Notes field, accepting any active input first, so that I can reach them quickly.
44. As a consultant, I want project names to be free text with the best match from earlier projects shown as dimmed ghost text, accepted with → at the end of the input, so that I type project names consistently.
45. As a consultant, I want Ctrl+C to copy the focused field's value to the system clipboard, so that I can reuse it elsewhere.
46. As a consultant, I want Ctrl+V to put the clipboard text into the focused field as accepted input (replacing any active input), so that pasting is one step.
47. As a consultant, I want pasted times like `12:35` accepted as `1235`, invalid times rejected with a message, and pasting onto a checkbox ignored, so that pasting never corrupts data.
48. As a consultant, I want Alt+Del to delete the focused period after confirming (Delete/Cancel), so that I can remove mistakes.
49. As a consultant, I want an implied break shown as its own row between two periods that don't follow each other directly, so that implied breaks are visible.
50. As a consultant, I want a period warning ⚠️ left of From for a period with an unusual time (before 06:00 or after 20:00) or a duration of 0h00m, so that such periods stand out.
51. As a consultant, I want the bottom of the right panel to show the actual expected working time, e.g. `Actual expected: 3h36m (regular 7h12m − 3h36m absence)`, with all absences grouped into one term, or just `Actual expected: 7h12m`, so that I understand my target for the day.
52. As a consultant, I want the right panel to scroll when periods do not fit, so that busy days remain usable.
53. As a consultant, I want long notes cut off with `…` and scrolled horizontally while editing, so that the layout stays intact.

### Consistency checks

54. As a consultant, I want the day to be checked when I leave it (Alt+↑/↓/PgUp/PgDn in the right panel, or ↑/↓/PgUp/PgDn in the left panel), and to stay on the day if the check fails, so that I never leave an inconsistent day behind.
55. As a consultant, I want periods sorted by From and completely empty periods removed automatically when I leave the day, so that I can add periods in any order and stray empty periods disappear.
56. As a consultant, I want the check to reject an empty From, overlapping periods, a From that is not before To, more than one sick, vacation or holiday period per day, an open period on a day other than today, and a project on a non-working period, so that my data stays valid.
57. As a consultant, I want non-consecutive periods to be allowed, so that implied breaks need no explicit break period.
58. As a consultant, I want a failed check to show a message inside the right panel and highlight the offending period, so that I know what to fix.

### Saving, quitting and undo

59. As a consultant, I want Ctrl+S to check all days and save, so that only consistent data is written by default.
60. As a consultant, I want a popup "<day>: <problem> (Go to day / Save anyway / Cancel)" when Ctrl+S finds a failing day, so that I can fix it or save anyway.
61. As a consultant, I want Ctrl+Q to check all days first, offering "Go to day / Quit anyway / Cancel" on failure, so that I do not quit with broken data by accident.
62. As a consultant, I want Ctrl+Q with unsaved changes to then ask "Save before quitting? (Save/Don't save/Cancel)", so that I do not lose work.
63. As a consultant, I want popup buttons selectable with ←/→ + Enter, with their first letter, and cancellable with Esc, so that popups are fast to use.
64. As a consultant, I want Ctrl+Z/Ctrl+Y to undo/redo accepted changes in the right panel, across all days of the session (jumping to the affected day), so that I can recover from mistakes.
65. As a consultant, I want saving to keep my undo history, so that I can still undo after saving.
66. As a consultant, I want a popup "File changed on disk (Overwrite/Cancel)" when the file was changed outside the application since loading, so that I do not overwrite changes from a sync tool.
67. As a consultant, I want the previous file kept as a `.bak` and the file written atomically, so that a crash never leaves me without data.
68. As a consultant, I want the application to exit with a clear error, leaving the file untouched, when the YAML cannot be parsed, so that a broken file is never overwritten.
69. As a consultant, I want my data stored in `~/Documents/timesheet.yaml` by default, overridable via a command-line argument, so that it is easy to find and back up.

### Appearance

70. As a consultant, I want the classic Norton Commander palette (blue background, cyan selection bar, input fields on a darker blue), so that the application feels familiar.
71. As a consultant, I want emoji to render as color emoji and still line up in columns in VTE terminals (GNOME Terminal, Ptyxis), Konsole, kitty, foot and ghostty, so that the UI looks good and stays aligned.

## Implementation Decisions

### Domain rules

- **Regular expected working time**: a constant 432 minutes (7h12m) for now, stored per day. For a day without a stored value (including empty days after the first recorded day) the constant applies. Vacation, sick and holiday periods never change it.
- **Actual expected working time**: computed, never stored: the regular expected working time minus the duration of the day's vacation, sick and holiday periods.
- **Worked time**: the total duration of a day's work periods. Breaks never count. Open periods count once To is filled in.
- **Day balance**: worked time minus actual expected working time. Shown for days from the first recorded day up to and including today.
- **Total balance**: sum of the day balances from the first recorded day up to and including that day. Not shown for future days or days before the first recorded day.
- **Balance format**: signed (`+0h45m`, `-1h12m`); zero has no sign (`0h00m`).
- **Day warning**: for days from the first recorded day (inclusive) up to but excluding today that are empty or have an open period.
- **Period types**: work, break, vacation, sick, holiday. Exactly one per period. In the UI the four non-working checkboxes are mutually exclusive, and none checked means work.
- **Times**: From and To are exactly four digits HHMM, 0000–2359. Periods never cross midnight.
- **Left panel display**: day numbers are padded to two characters (`ma  2 nov`) so columns align. A day with only non-working periods shows `0h00m` worked time; a day without periods shows nothing.
- **Consistency checks** (not exhaustive): From is not empty; no overlaps, and nothing after an open period; From before To; at most one sick, one vacation and one holiday period per day; open period only on the current day; non-working periods have no project. Before checking a day, completely empty periods are removed and the rest are sorted by From, so ordering never fails a check.
- **Check triggers**: leaving a day (B) checks that day, but only if it was changed in this session, so you can always browse past broken days. Ctrl+S and Ctrl+Q first sort all days and remove empty periods (which alone isn't an unsaved change), then check all days; the popup names the earliest failing day, and "Go to day" selects it, focuses the right panel and shows the problem there. On Ctrl+Q the consistency popup comes before the "Save before quitting?" popup.
- **Check results**: messages name periods by their times (e.g. `0900–1200 overlaps 1130–1300`, `Open period 1230– on a past day`, `More than one sick period`); only the first problem is shown. The offending periods get a red background until the next key press. Every day that currently fails a check gets a day warning.
- **Shortcut modifiers** used in the original design: (A) not available while a field is being edited; (B) runs a consistency check and cancels the action if it fails; (C) accepts the active input first.
  - Alt+↑/↓/PgUp/PgDn in the right panel: A, B (ignored while editing; they don't accept active input). ←/→ in the right panel: previous/next field of the period, stopping at its edges; while editing they move the cursor. ↑/↓ in the right panel: A. Alt+b/v/s/h/p/n: C. Alt+←/→: A. Ctrl+Q: A, B. Ctrl+S: A, B (all days). Ctrl+Z/Y: A. Alt+Del, Alt+Enter: A.
- **Editing**: From/To take only digits, at most 4; an empty value may be accepted. Enter with an invalid time keeps the field in edit mode and shows a message. Tab/Shift-Tab accept and move on (staying put if accepting fails). All other non-editing shortcuts are ignored while editing. Typing on a checkbox does nothing.
- **Messages**: one-line notices in the right panel (invalid time, failed consistency check, rejected paste); a message disappears at the next key press. The right panel always reserves a row for them.
- **Active input display**: while editing, the field shows the text around the cursor; `…` only appears outside edit mode.
- **Adding periods**: Alt+Enter adds a period from any field or without focus, but not while the last period is open (no To yet), which includes an empty last period. Enter no longer adds periods; Space no longer toggles checkboxes.
- **Delete popup**: Delete is the default button.
- **Period types**: unchecking an absence empties From and To only if the period is still the day's only one and they still hold the auto-filled full day (0900 and 0900 + regular expected working time); the values decide, so hand-typed times equal to a full day are emptied too (accepted; undo covers it). Switching from one absence (or to Break) keeps From and To, without emptying or auto-fill. Alt+b/v/s/h/p/n need a focused period in the right panel; they accept active input first and do nothing if that fails.
- **Deleting**: after deleting a period, focus moves to the same field of the period taking its place (the previous one if the last was deleted); no focus if the day becomes empty.
- **Tab order** within a period follows the fields on screen: From, To, Project, Break, Vacation, Sick, Holiday, Notes. Duration is not focusable. Tab with nothing focused focuses From of the first period.
- **Right panel layout**: period warning ⚠️ left of From (unusual time or 0h00m); then From, To, Duration, Project, the four checkboxes (emoji headers ☕ 🏖️ 🤒 🎄) and Notes, which takes the remaining width. An empty row separates periods; an implied break is its own row in plain text with Break checked, not focusable.
- **Look** (Midnight Commander): light gray text on blue, single-line borders, yellow column headers, the active panel's title black on light gray. Selection: cyan bar with black text; in an inactive left panel the selected day's row is cyan text without a bar. Left panel title `Days: <year>`; the year separator shows the new year centered. Fields and checkboxes are light gray boxes (a checked box shows ✓), cyan when focused or being edited. Popups: light gray with a black shadow, a short title in the frame (`Delete period`, `Quit`, `File changed`, `Consistency`), the message inside (wrapped), solid buttons black on white, the selected one black on cyan, hotkey letter in blue.
- **Panel focus**: the active panel's title is highlighted; in an inactive left panel the selected day's row is cyan text without a bar. Leaving and re-entering the right panel on the same day keeps the focused field; selecting another day clears it. Shift-Tab with nothing focused focuses the last field (Notes) of the last period.
- **Right panel title**: shows the selected day without padding, e.g. `Periods: ma 2 nov`; popups use the same unpadded date.

### Undo

- One session-wide history covering all days. One step is one accepted change: an accepted field value, a paste, a checkbox toggle (including any From/To auto-fill it triggers), adding a period, or deleting a period.
- The automatic sort when leaving a day is not an undo step.
- Undoing a change on another day selects that day.

### Storage

YAML file, default `~/Documents/timesheet.yaml`, overridable with a single command-line argument (path).

```yaml
days:
  "2026-11-12":
    regular_expected_working_time_in_minutes: 432
    periods:
      - from: "12:35"
        to: null          # open period
        project: "otto"
        notes: null
        type: work        # work | break | vacation | sick | holiday
```

- A missing or empty file is an empty timesheet; the file is created on the first save.
- Unknown keys and invalid `HH:MM` times are parse errors: the application exits without touching the file.
- Empty To, project and notes are written as `null`. `""` is read as `null`.
- `type` is always written.
- Days without periods are not written. `regular_expected_working_time_in_minutes` is written when a day gets its first period, and existing values are kept. A day in the file without it is a parse error.
- Writes are atomic: temp file, flushed to disk, renamed over the old file, keeping its permissions. A failed save removes the temp file. Comments in the file are not preserved.
- The first save of a session keeps the file as it was when the application started as `timesheet.yaml.bak`; later saves in the same session leave it alone.
- If the file path is a symlink, it is followed: the file it points to is replaced, the symlink stays, and the temp file and backup go next to the target.
- Before writing, the application detects whether the file changed on disk since it was loaded or last saved.
- Dates, times, project and notes are always written double-quoted, so YAML 1.1 readers don't take `08:45` for a number or `2026-11-02` for a timestamp. The application writes the YAML itself for this; reading still uses the YAML library.
- Ctrl+Q with unsaved changes asks "Save before quitting?"; choosing Save may lead to "File changed on disk", after which Overwrite quits and Cancel stays in the application. A failed save shows a message in the right panel.

### Modules

- **Timesheet domain** (pure, no I/O): days, periods, period types, HHMM parsing/formatting, derived figures (worked time, actual expected working time, day balance, total balance), consistency checks, day and period indicators. The deepest module; most rules live here.
- **Timesheet store**: loads and saves the YAML file. Interface: load a path into a timesheet (or a parse error); save a timesheet, refusing when the file changed since a given fingerprint unless overwriting is forced.
- **Application state**: the whole interactive state (screen, panel focus, selected day, focused period/field, active input, popups, undo/redo history, dirty flag, messages). Takes key events and produces state changes plus effects (save, quit, clipboard). Today/now, clipboard and file system reach it through small injected ports so tests can fake them.
- **Rendering**: draws the application state with Norton Commander styling. After each frame, it paints the second cell of every wide symbol with a space in that cell's background, then redraws the symbol on top of it. Otherwise Ptyxis leaves half the background of 🏖️/⚠️ unpainted (ADR-0001). This approach was proven in the prototype:

  ```rust
  // from the prototype: run after terminal.draw() on the completed buffer
  if cell.symbol().width() == 2 {
      queue!(out, SetBackgroundColor(bg), SetForegroundColor(fg),
             MoveTo(x + 1, y), Print(" "), MoveTo(x, y), Print(cell.symbol()))?;
  }
  ```

### Technology (implementer's choice, not a decision by the user)

ratatui + crossterm for the TUI, serde + serde_yaml_ng for YAML (serde_yaml is unmaintained), arboard for the clipboard, jiff for dates and times. The Rust toolchain is added to the existing mise configuration as `rust = "latest"`.

## Testing Decisions

- Good tests check external behavior only: given input, they check what the user sees or what is persisted, never internal structure.
- **Seam 1, Timesheet domain** (test-first, red-green-refactor): balances, actual expected working time, consistency checks, HHMM parsing, sorting and indicators, tested through the domain module's public interface.
- **Seam 2, Application** (the highest seam; tests written alongside the code, not strictly test-first): feed key sequences into the application state with a fake clock, fake clipboard and in-memory file system, render to ratatui's TestBackend, and check the rendered buffer, the persisted YAML and clipboard contents. This covers navigation, editing, popups, undo/redo and saving end-to-end.
- The store's YAML round-trip (null handling, type field, atomic write, `.bak`, change detection) is covered through seam 2's in-memory file system. A small set of tests uses a real temporary directory for the atomic write.
- Emoji alignment in real terminals can't be checked with TestBackend. It is proven by the throwaway prototype (see Further Notes).
- A sample timesheet file with realistic weeks (vacation, sick and holiday periods, an open period, an empty day) serves demos and tests, before periods can be created in the UI.
- No prior art: the repository has no code yet.

## Out of Scope

- Moco synchronization (later milestone). Project names stay free text and will be mapped to Moco then.
- A managed project list.
- Screens other than Tracking and Help.
- Weekend days, periods crossing midnight, configurable regular expected working time.
- Auto-save, mouse support.
- Verifying emoji rendering in terminals other than Ptyxis, and installing those terminals.

## Further Notes

- **Emoji prototype (done)**: in Ptyxis, ratatui aligns 🏖️/⚠️ on its own. Only the background needed fixing, which the approach above does (variant 5). Raw printing without ratatui misaligns. Prototype code: branch `prototype/emoji-alignment` (commit 383c813).
