# Time tracking (milestone 1)

Status: ready-for-agent

## Problem Statement

As a consultant, I record my working time by hand in a free-form text file. Adding up worked time per day, keeping track of how far I am ahead or behind my actual expected working time, and spotting gaps or mistakes (overlaps, forgotten days, open periods) is manual, slow and error-prone. Later, the same data must be booked in Moco, which today means retyping it.

## Solution

A keyboard-driven terminal application (TUI) in Rust, with the look and feel of Midnight Commander and the polish of a modern TUI. It needs a terminal with the Kitty keyboard protocol, such as Ghostty (ADR-0003). Its first screen, **Tracking**, has two panels:

- **Left panel**: an endless list of days (Monday–Friday) with worked time, day balance, total balance and indicators. It shows the selected day; it has no keys of its own.
- **Right panel**: the day note and the periods of the selected day, edited in place, with consistency checks and indicators. A field in it always has focus.

Data is stored in a human-readable YAML file. Moco synchronization follows in a later milestone.

## User Stories

### Starting and the screen

1. As a consultant, I want the application to open on the Tracking screen with today selected, so that I can start recording immediately.
2. As a consultant, I want the previous Friday to be selected when I start the application on a weekend, so that I land on the most recent working day.
3. As a consultant, I want `tijdje --help` to print the usage and `tijdje --version` the version, and an unknown option to be an error, so that the command line explains itself.
4. As a consultant, I want the application to refuse to start, with a message naming supported terminals, when my terminal doesn't support the Kitty keyboard protocol, so that I never end up with shortcuts that silently don't work.
5. As a consultant, I want screens switched with function keys and a function-key bar across the full width of the bottom line (F1 Help, F2 Track), so that navigation feels like Midnight Commander.
6. As a consultant, I want F1 to open a Help window listing all shortcuts in groups (General, Day navigation, Field navigation, Field editing, Period definition, Period management), closed with Esc or F1, so that I can look up keys quickly.
7. As a consultant, I want a "terminal too small" message when my terminal is smaller than 120×24, so that the layout never breaks silently.
8. As a consultant, I want "today" to stay correct while the application runs past midnight, so that balances and warnings stay right.

### Left panel (days)

9. As a consultant, I want to see a vertical list of days with the columns date, worked time, day balance, total balance, absence indicator and day warning, so that I see my status at a glance.
10. As a consultant, I want exactly one day to be selected at all times, shown with a cyan bar, so that the right panel always has a clear context.
11. As a consultant, I want only Monday–Friday to be listed, with a single-line separator between weeks and a double-line separator with the new year between years, so that weeks and years are visually distinct.
12. As a consultant, I want the list to scroll endlessly into the past and the future, so that I can plan vacation and review history.
13. As a consultant, I want dates in the Dutch format "ma 12 nov", so that they match how I think about my week.
14. As a consultant, I want durations like `1h55m`, `42m` for less than an hour, and balances signed like `+45m` and `-1h12m` (zero is `0m`), so that values are short and unambiguous.
15. As a consultant, I want the total balance to add up from the first recorded day, with an empty weekday after it counting with its full regular expected working time, so that forgotten days lower my balance until I record them.
16. As a consultant, I want days before the first recorded day to show no balances and no warnings, and future days no balances, so that history before I started tracking and planned absences are not noise.
17. As a consultant, I want an absence indicator per day (🤒 sick, 🏖️ vacation, 🎄 holiday; at most one, in that priority), so that I see absences at a glance.
18. As a consultant, I want a day warning ⚠️ for a day that fails the consistency check and for an empty past weekday after the first recorded day, so that I notice broken and forgotten days.

### Day navigation

19. As a consultant, I want Ctrl+↑/↓ to select the previous/next day and Ctrl+PgUp/PgDn the previous/next week, so that I can move quickly.
20. As a consultant, I want Ctrl+G to open a "Go to date" dialog where I type a date (`2026-11-15`, `15 nov`, `15 nov 2025`, Dutch month abbreviations) or pick today, so that I can jump to any day.

### Right panel (day note and periods)

21. As a consultant, I want a day note as a full-width field with its own header at the bottom of the right panel, above the bottom line, so that I can attach a remark to the day.
22. As a consultant, I want to see the periods of the selected day with an indicator, From, To, Duration, Project, the checkboxes Break, Free, Vacation, Sick and Holiday, and Notes, so that I can record what I did.
23. As a consultant, I want a day without periods to show one empty period I can fill in, so that I can start a day without an extra key.
24. As a consultant, I want a field to have focus at all times, From of the first period after switching days, so that I can always start typing.
25. As a consultant, I want From and To entered as four digits HHMM without a colon, so that typing times is fast.
26. As a consultant, I want a period to have a part of day instead of From and To (full day, 1st half, 2nd half of the regular expected working time), shown as "Full day", "1st half" or "2nd half", so that whole and half days need no times.
27. As a consultant, I want the duration of each period computed automatically, so that I do not have to calculate it.
28. As a consultant, I want an implied break shown as its own row between two from–to periods that don't follow each other directly, and an empty row between periods, so that the day is easy to read.
29. As a consultant, I want an indicator left of From: ⚠️ for periods involved in the day's consistency problem, otherwise ❗ for a period warning, otherwise the emoji of a non-working period type (☕ also on implied-break rows), so that I see at a glance what needs attention and what a period is.
30. As a consultant, I want a period warning ❗ for an unusual time (before 06:00 or after 20:00) or a period that directly follows the previous one with the same project, period type and notes, so that likely mistakes stand out.
31. As a consultant, I want the bottom of the right panel to show `Worked [6h12m] of [7h12m] | Work until [17:42] for [8h]`, so that I know where I stand and until when to work for 8h.
32. As a consultant, I want the right panel to scroll when periods do not fit, and long notes cut off with `…`, so that busy days remain usable.

### Field navigation and editing

33. As a consultant, I want ←/→/↑/↓ to move focus to the field left/right/above/below, PgUp/PgDn to move 10 periods, Home/End to go to the first/last field of a period, and Tab/Shift-Tab to walk through all fields in order, so that I can move freely.
34. As a consultant, I want Ctrl+Enter to move to the next period's From, and in the last period to add a new period whose From is the previous period's To, so that consecutive periods are quick to enter.
35. As a consultant, I want typing in a field to replace its value, Enter to start editing with the existing text, Enter again to accept and Esc to reject, so that I stay in control of every change.
36. As a consultant, I want Enter on a checkbox to toggle it, so that checkboxes work like other fields.
37. As a consultant, I want Del to clear a field (uncheck a checkbox, switch a part of day back to from–to), so that clearing is one key.
38. As a consultant, I want shell-like editing keys while editing (Home/Ctrl+A, End/Ctrl+E, Ctrl+Backspace/Ctrl+W, Ctrl+U, Ctrl+K, Ctrl+←/→), so that editing feels familiar.
39. As a consultant, I want Ctrl+C/Ctrl+V to copy and paste a field's value through the system clipboard, so that I can reuse values.
40. As a consultant, I want changing a period's To to also change the next period's From when that was equal to the old To, so that consecutive periods stay connected.
41. As a consultant, I want project names to be free text with the best match from earlier projects shown as dimmed ghost text, taken with → (keep editing), Tab or Ctrl+Enter (accept and move on) at the end of the input while editing, so that I type project names consistently.

### Period definition and management

42. As a consultant, I want Alt+b/f/s/v/h to toggle Break/Free/Sick/Vacation/Holiday, with the checkboxes mutually exclusive, so that I can set the period type from anywhere in the period.
43. As a consultant, I want a Free period type (🆓) for time I take off from my overtime, which neither counts as worked time nor reduces the actual expected working time, so that I can record working less on purpose.
44. As a consultant, I want Alt+0 to switch a period to from–to and Alt+1/2/3 to full day, 1st half and 2nd half, so that I can switch quickly.
45. As a consultant, I want checking Vacation, Sick or Holiday on the day's only empty period to set it to full day, and unchecking to undo that, so that whole-day absences take one key.
46. As a consultant, I want Ctrl+Del to delete the focused period after confirming, so that I can remove mistakes.

### Consistency checks

47. As a consultant, I want the day to be checked when I leave it after changing it, and to stay on the day if the check fails, so that I never leave an inconsistent day behind.
48. As a consultant, I want periods sorted (full day, 1st half, from–to periods by From, 2nd half) and empty periods removed when I leave a day, so that I can add periods in any order.
49. As a consultant, I want the check to reject a period without From or part of day, overlapping from–to periods, anything after an open period, a From not before To, more than one sick, vacation or holiday period, an open period on a day other than today, a project on a non-working period, a part of day on a break, and more than one full day or more than one of each half, so that my data stays valid.

### Saving, quitting, undo and notifications

50. As a consultant, I want Ctrl+S to check all days and save, with "Go to day / Save anyway / Cancel" when a day fails, and a "Saved" notification afterwards, so that I know my data is written.
51. As a consultant, I want Ctrl+Q to check all days first and to ask "Save before quitting?" when there are unsaved changes, so that I don't lose work.
52. As a consultant, I want Ctrl+Z/Ctrl+Y to undo/redo accepted changes across all days of the session, so that I can recover from mistakes.
53. As a consultant, I want messages (saved, invalid time, failed check, clipboard, nothing to undo) shown as a global notification, centered with a cyan background, until my next key, so that I always see them in the same place.
54. As a consultant, I want a dialog when the file changed on disk, the previous file kept as `.bak`, the file written atomically, and a clear error for a broken or unsupported file, so that I never lose data.
55. As a consultant, I want my data stored in `~/Documents/timesheet.yaml` by default, overridable via a command-line argument, so that it is easy to find and back up.

### Appearance

56. As a consultant, I want the Midnight Commander look (light gray on blue, cyan selection, light gray fields, yellow headers), so that the application feels familiar.
57. As a consultant, I want emoji to render as color emoji and line up in columns, so that the UI looks good and stays aligned.

## Implementation Decisions

### Domain rules

- **Regular expected working time**: a constant 432 minutes (7h12m) for now, stored per day. For a day without a stored value (including empty days after the first recorded day) the constant applies. Absences never change it.
- **Actual expected working time**: computed, never stored: the regular expected working time minus the duration of the day's absences.
- **Worked time**: the total duration of a day's work periods. Breaks and free periods never count. Open periods count once To is filled in.
- **Day balance**: worked time minus actual expected working time. Shown for days from the first recorded day up to and including today; an empty weekday in that range counts with its full regular expected working time. Not shown for future days or days before the first recorded day.
- **Total balance**: the sum of the day balances from the first recorded day up to and including that day. Not shown for future days or days before the first recorded day.
- **Durations**: `1h55m`; under an hour `42m` (padded where columns align); balances signed (`+42m`, `-1h12m`); zero is `0m`.
- **Period types**: work, break, free, vacation, sick, holiday. Exactly one per period; the five non-working checkboxes are mutually exclusive, none checked means work. Free is a non-working period that is not an absence: it neither counts as worked time nor reduces the actual expected working time. Otherwise free periods follow the general rules: a gap between a free period and another from–to period is an implied break; adjacent free periods with the same notes get the needless-split ❗; free periods outside 06:00–20:00 get the unusual-time ❗. Free time beyond the available overtime isn't checked; the total balance just goes negative.
- **Times**: From and To are exactly four digits HHMM, 0000–2359. Periods never cross midnight.
- **Part of day**: instead of From and To, a period can have a part of day: full day (100% of the regular expected working time), 1st half (half of it, rounded down to the minute) or 2nd half (the rest, so both halves add up to exactly a full day). Allowed for work, free, vacation, sick and holiday, not for break. A part-of-day period has no times; its duration counts like any other period's in all calculations (worked time, absences, balances, "Worked … of …", "Work until").
- **Sort order**: full day, 1st half, from–to periods by From, 2nd half. Empty periods are removed when sorting.
- **Implied breaks and overlaps**: only between from–to periods.
- **Open period**: a from–to period with a From and no To. A part-of-day period is never open.
- **Empty period**: no times, no part of day, no project, no notes, and type work.
- **Empty day**: a day without periods, or with only empty periods, counts as empty everywhere: no worked time in the left panel, not a recorded day for the first recorded day, its full regular expected working time in the balances, and a day warning as an empty past weekday. It is not written on save, unless it has a day note: a day with only a day note is written but otherwise counts as empty.
- **Chained times**: changing a period's To also changes the next from–to period's From if that was equal to the old To. Changing a From changes nothing else. Decided: "next" is the next period on screen; clearing a To leaves the next From unchanged; moving a To past the next period's To still chains, and the consistency check flags the result; filling in the To of an open period doesn't chain (there was no old To); part-of-day and empty periods are skipped when looking for the next from–to period.
- **Consistency checks** (not exhaustive): a period has a From (To may be empty only today) or a part of day; no overlaps between from–to periods, and nothing after an open period; From before To; at most one sick, one vacation and one holiday period per day; open period only on the current day; work periods have a project (also an open period on the current day), and non-working periods have none; no part of day on a break; at most one full day, and at most one 1st half and one 2nd half, never a full day together with a half. These last checks come after the existing ones; their messages are `A break can't be a part of day (Full day)`, `More than one full day`, `More than one 1st half` / `More than one 2nd half` and `Full day together with a half`. Part-of-day periods are named by their label ("Full day", "1st half", "2nd half") in messages.
- **Check triggers**: leaving a day checks it, but only if it was changed in this run of the application, so broken past days can always be browsed; if the check fails, the day stays selected. Before checking, the day is sorted and its empty periods removed. Ctrl+S and Ctrl+Q sort all days (which alone isn't an unsaved change) and check all days; for the earliest failing day, a dialog offers Go to day / Save anyway (or Quit anyway) / Cancel. "Go to day" selects that day and shows the problem. On Ctrl+Q, the consistency dialog comes before "Save before quitting?".
- **Check results**: only the first problem is shown, as a global notification that names periods by their times (e.g. `0900–1200 overlaps 1130–1300`, `Open period 1230– on a past day`, `More than one sick period`). The offending periods get a red background until the next key press.
- **Warnings vs. consistency failures**: a period warning ❗ marks a likely mistake that is allowed: an unusual time, or a from–to period that directly follows the previous from–to period in time (its From equals that period's To) with the same project, period type and notes, also when both have no project and no notes (the warning goes on the second one). A consistency failure ⚠️ marks a rule violation; a period of 0m (From equals To) is one ("From is not before To"). Warnings and "Work until" use the periods in time order (a sorted copy), not the order on screen.
- **Period indicator** (left of From, one symbol): ⚠️ for periods involved in the day's current consistency problem (determined live, on a sorted copy of the day without empty periods, mapped back to the periods on screen; it shows right away, also while the day is being filled in), else ❗ for a period warning, else the emoji of a non-working period type (☕ break, 🆓 free, 🏖️ vacation, 🤒 sick, 🎄 holiday), else nothing. Implied-break rows show ☕ in the same column.
- **Day warning** (left panel, ⚠️ only): a day that fails the consistency check, or an empty past weekday after the first recorded day. A day with only a day note counts as empty here, and doesn't count as a recorded day for the first recorded day. ❗ is never shown in the left panel. An empty past day is not a consistency failure: saving and quitting don't ask about it.
- **Absence indicator** (left panel): 🤒 > 🏖️ > 🎄; free periods have no absence indicator.
- **"Work until"**: the time until which to work on the selected day to reach 8h worked (a fixed value for now), given the periods as they are: base + (8h − worked time) + missing break time. The base is the latest To of the from–to periods, or the From of an open last period. The missing break time is 1h minus the break time so far (explicit breaks plus implied breaks; free periods don't count) if that is under 1h, otherwise nothing. Shows `[–]` when 8h is already worked, the day has no from–to periods, or the result would be 24:00 or later. Example: 0900–1200 and 1230–1500 is 5h30m worked with a 30m implied break: 1500 + 2h30m + 30m = `[18:00]`.
- **Bottom line**: `Worked [<worked time>] of [<actual expected working time>] | Work until [<time>] for [8h]`; the bracketed parts are spans with a cyan background.

### Screen and panels

- **No panel focus**: the left panel only shows the selected day (cyan bar) and has no keys; focus is always in the right panel. Panel titles have no highlight. Titles: `Days: <year>` and `Periods: <unpadded date>`.
- **Right panel**: the header and the period rows; then, at the bottom, the day note: a header row "Day note" styled like the period column headers, with the day note's full-width input field on the line below; then an empty line and the bottom line. The day note's header and input field have the same left padding as the bottom line. A day without periods shows one empty period; it becomes a real period once something is filled in, and an empty period is never saved.
- **Focus**: a field always has focus. Switching days focuses From of the first period. After deleting the day's only period, the empty period's From has focus.
- **Field order** (Tab order, and the basis for arrow navigation): per period: From, To (or the part-of-day field), Project, Break, Free, Vacation, Sick, Holiday, Notes; then the day note. Duration and implied-break rows are not focusable.
- **Part-of-day field**: replaces From and To on screen ("Full day", "1st half", "2nd half"), 9 cells wide (From, the gap and To), and is one field; the Duration column shows its duration. Enter and typing do nothing on it. For ↑/↓, it counts as standing above or below both From and To; leaving it with ↑/↓ towards a from–to row lands on From.
- **Checkbox columns**: emoji headers ☕ 🆓 🏖️ 🤒 🎄; boxes are light gray, ✓ when checked, cyan when focused.
- **Global notifications**: one-line messages (saved, invalid time, failed check, clipboard, nothing to undo/redo) centered across the screen with a cyan background, floating over the row just above the panels' bottom border, until the next key press. They replace the right panel's message row. "Saved" isn't shown when saving while quitting.
- **Help window**: shown over the Tracking screen; grouped (General, Day navigation, Field navigation, Field editing, Period definition, Period management); colors as in Midnight Commander's user menu: cyan background, light text, yellow title, single-line frame, black shadow; scrolls if it doesn't fit; closed with Esc or F1.
- **Function-key bar**: ten slots stretched evenly over the full width; F1 Help, F2 Track (more screens will follow).
- **Dialogs** (Delete period, Quit, File changed, Consistency, Go to date): light gray background with black text and a black shadow; a short title in the frame (blue) and the message inside, wrapped; solid buttons, black on white, the selected one black on cyan, the hotkey letter in blue. ←/→ and Enter, or a button's first letter, choose a button; Esc cancels. "Save before quitting?" offers Save / Don't save / Cancel; "File changed on disk" offers Overwrite / Cancel; the Delete dialog's default button is Delete.
- **Dialogs with input fields** (like Midnight Commander's dialogs): Tab/Shift-Tab move focus between the input fields and the buttons; typing goes to the focused input field; with an input field focused, Alt and a button's highlighted letter choose that button; with a button focused, the plain letter works too; ←/→ move between buttons; Enter in an input field presses the default button, on a button that button; Esc cancels.
- **Look**: Midnight Commander: light gray text on blue, single-line borders, yellow column headers, cyan selection bar, light gray fields (cyan when focused or editing).

### Keyboard

The terminal must support the Kitty keyboard protocol (ADR-0003); the application enables disambiguated key codes on start and restores the terminal on exit and on a panic.

- **General**: F1 Help window; F2 Track; Ctrl+S save; Ctrl+Q quit; Ctrl+Z/Ctrl+Y undo/redo; Esc rejects the active input or closes a dialog.
- **Day navigation**: Ctrl+↑/↓ previous/next day; Ctrl+PgUp/PgDn previous/next week; Ctrl+G go to date. Leaving a changed day runs its consistency check.
- **Field navigation**: ←/→/↑/↓ move to the field left/right/above/below (←/→ stop at the row's edges; ↑/↓ go to the closest field in the row above/below, skipping implied-break rows; the day note is below the last period: ↓ from the last period goes to the day note, ↑ from the day note goes to From of the last period); Tab/Shift-Tab (without Ctrl or Alt) walk through all fields and wrap around within the day (after the day note comes From of the first period, and the other way around); PgUp/PgDn move 10 periods up/down, keeping the field and stopping at the first period and at the day note (which counts as the row after the last period); from the day note, PgUp goes to From of the period 10 above it, or of the first period; Home/End (not editing) move to the first/last field of the focused period (From/Notes); Ctrl+Enter moves to the next period's From, and in the last period adds a period after it (not while the last period is open), focused on its From, which is the previous period's To (empty after a part-of-day period). Ctrl+Del does nothing on the placeholder empty period of a day without periods. When the last period is open, Ctrl+Enter shows the notification "The last period is still open"; on the placeholder empty period it does nothing. Ctrl+Enter while editing the placeholder empty period first accepts the input, which makes it a real period; the rule for an open last period then applies. On the day note, ↓, Ctrl+Enter, Home/End, the period-definition keys (Alt+b/f/s/v/h, Alt+0..3), Ctrl+Del and ←/→ do nothing.
- **Field editing**: Enter toggles a checkbox, starts editing an input field with its existing text (cursor at the end), or accepts the active input; typing replaces an input field's value (also Notes); Del (not editing) clears an input field, unchecks a checkbox, or switches a part of day back to from–to; Ctrl+C/Ctrl+V copy/paste. While editing: ←/→ move the cursor, Home/Ctrl+A and End/Ctrl+E jump to the start/end, Backspace/Del delete a character, Ctrl+Backspace/Ctrl+W delete the previous word, Ctrl+U deletes to the start, Ctrl+K to the end, Ctrl+← moves to the start of the previous word and Ctrl+→ to the end of the next word (as in the shell). A word is anything between spaces.
- **Project suggestion**: of the projects anywhere in the timesheet that start with the active input (a prefix match, ignoring case) and are longer, the most used one; ties go to the most recently used. Projects differing only in case ("Acme", "ACME") are separate projects. It shows as dimmed ghost text only while the cursor is at the end of a non-empty active input in Project, also right after Enter starts editing an existing project that a longer project starts with. → there fills in the suggestion, in the earlier project's spelling, and keeps editing. Tab/Shift-Tab and Ctrl+Enter take the suggestion, accept and move on. Enter accepts only the typed text, and End and Ctrl+→ keep their editing meaning.
- **Period definition**: Alt+b/f/s/v/h toggle Break/Free/Sick/Vacation/Holiday; Alt+0 switches a part-of-day period to from–to mode (empty times; on a from–to period it does nothing); Alt+1/2/3 switch to full day / 1st half / 2nd half. They accept the active input first and do nothing if that fails. After Alt+1/2/3 focus is on the part-of-day field; after Alt+0, or Del on the part-of-day field, on From.
- **Period management**: Ctrl+Del deletes the focused period after confirming (Delete is the default button); not while editing.
- **While editing**: Tab/Shift-Tab and Ctrl+Enter accept and move on (staying put if accepting fails); the period-definition keys (Alt+b/f/s/v/h, Alt+0..3) accept first; other navigation and management keys (day navigation, Ctrl+Del, Ctrl+G, ↑/↓) are ignored, and so are Ctrl+S, Ctrl+Q, F1 and Ctrl+Z/Ctrl+Y, silently.
- **Go to date dialog**: an input field and the buttons Go, Today and Cancel. Accepts `YYYY-MM-DD`, `DD MMM` and `DD MMM YYYY` with Dutch month abbreviations (`jan feb mrt apr mei jun jul aug sep okt nov dec`); without a year, the current year. A weekend date selects the Friday before. An invalid date shows a notification. The input field starts empty and has focus; it follows the rule for dialogs with input fields, with Go as the default button. An invalid or empty date shows a notification and keeps the dialog open. Today on a weekend selects the Friday before. Only the three formats are accepted (`2026-1-5`, `15nov` and `15 nov.` are not); the month may be typed in any case and the day as one or two digits; years 0001–9998. Ctrl+C/Ctrl+V copy and paste in the input field as in other input fields. If the chosen day is already visible in the day list, the list scrolls as with other day navigation; otherwise it is laid out around the day as on startup.
- **Ghostty**: Ctrl+Enter is Ghostty's full-screen key; the README documents `keybind = ctrl+enter=unbind`. Ctrl+PgUp/PgDn and Alt+1..8 switch Ghostty tabs, but come through in a window with a single tab.
- Removed: Alt+←/→, Alt+↑/↓, Alt+PgUp/PgDn, Alt+Enter, Alt+Del, Alt+p/n, and the left panel's own keys.

### Editing details

- From/To take only digits, at most 4; an empty value may be accepted. Enter with an invalid time keeps the field in edit mode and shows a notification. Typing on a checkbox does nothing.
- While editing, the field shows the text around the cursor; `…` only appears outside edit mode.
- **Clipboard**: Ctrl+C copies the focused field's value, or the active input while editing; Ctrl+V replaces the field with the clipboard text as accepted input. Pasted times are accepted as `HHMM`, `HH:MM` or `H:MM`; anything else is rejected with a notification. Line breaks become spaces. Checkboxes and the part-of-day field can't be copied or pasted.
- **Period types**: checking an absence (with Enter, Alt or by any other means) on the day's only period, when it has no times and no part of day (notes may be filled in), sets it to full day; unchecking it (with Enter, Alt or Del) removes the full day again if it is still the only period and still a full day, keeping its notes. A project on a non-working period is a consistency failure as always. Switching between non-working types keeps the times or part of day. Checking an absence never fills in times (the earlier 0900–… auto-fill is gone), and unchecking never clears times.

### Undo

- One session-wide history covering all days. One step is one accepted change: an accepted field value (including the day note and a chained From), a paste, a checkbox toggle (including any auto-fill), a part-of-day switch, adding a period, deleting a period, or clearing a field.
- Sorting a day is not an undo step. Undoing a change on another day selects that day; on the same day, focus stays at the same position if that period exists. Undo/redo don't work on Help, while editing or with a dialog open; "Nothing to undo/redo" otherwise.

### Clock

- "Today" comes through an injected clock and is checked every minute while the application runs (the event loop polls with a timeout). After midnight, balances and warnings update; the selected day stays.

### Command line

- `tijdje [PATH]`: PATH defaults to `~/Documents/timesheet.yaml`.
- `--help` prints the usage (including the default path) and exits; `--version` prints the version and exits; any other option is an error with a non-zero exit code.
- Every argument starting with `-` is an option, so a path starting with `-` is given as `./-file.yaml`. Two or more paths are an error. A wrong invocation exits with code 2, a file that can't be loaded with 1.
- The schema version is checked before the rest of the file is parsed, so a newer file with new keys still gets the clear version message; a version that isn't a whole number is refused the same way.

### Storage

YAML file, default `~/Documents/timesheet.yaml`.

```yaml
schema_version: 1
days:
  "2026-11-12":
    regular_expected_working_time_in_minutes: 432
    notes: "Team day"   # day note, or null
    periods:
      - from: "12:35"
        to: null          # open period
        part_of_day: null # full_day | first_half | second_half (then from/to are null)
        project: "otto"
        notes: null
        type: work        # work | break | free | vacation | sick | holiday
```

- `schema_version` is always written; a file without it is version 1; any other version than 1 makes the application refuse to start with a clear message. The version increases only on breaking changes; the day note, free and part of day are additions, so it stays 1.
- A missing or empty file is an empty timesheet; the file is created on the first save.
- Unknown keys and invalid `HH:MM` times are parse errors: the application exits without touching the file.
- Empty To, project, notes, day notes and part of day are written as `null`. `""` is read as `null`. A day note of only spaces, typed or pasted, is no day note (`null`).
- `type` is always written.
- `part_of_day` together with a non-null `from` or `to` is a parse error.
- Days with only empty periods (or none) and without a day note are not written. `regular_expected_working_time_in_minutes` is written for every written day, and existing values are kept. A day in the file without it is a parse error.
- Writes are atomic: temp file, flushed to disk, renamed over the old file, keeping its permissions. A failed save removes the temp file. Comments are not preserved.
- The first save of a session keeps the file as it was when the application started as `timesheet.yaml.bak`.
- Symlinks are followed: the target is replaced, the symlink stays, and the temp file and backup go next to the target.
- Before writing, the application detects whether the file changed on disk since it was loaded or last saved.
- Dates, times, project and notes are written double-quoted; the application writes the YAML itself.

### Modules

- **Timesheet domain** (pure, no I/O): days, periods, period types, parts of day, time parsing/formatting, derived figures (worked time, actual expected working time, balances, "Work until"), consistency checks, warnings and indicators. The deepest module; most rules live here.
- **Timesheet store**: loads and saves the YAML file, including the schema version.
- **Application state**: the whole interactive state (selected day, focused field, active input, dialogs, notifications, undo/redo history, unsaved changes). Takes key events and produces state changes plus effects. The clock, clipboard and file system reach it through small injected ports so tests can fake them.
- **Rendering**: draws the application state in the style of Midnight Commander, including the wide-symbol background fix (ADR-0001).
- **Terminal setup**: checks for and enables the Kitty keyboard protocol, and restores the terminal on exit and on a panic (ADR-0003).

### Technology (implementer's choice, not a decision by the user)

ratatui + crossterm for the TUI, serde + serde_yaml_ng for YAML (serde_yaml is unmaintained), arboard for the clipboard (without its image support, with Wayland support via `wayland-data-control`), jiff for dates and times. The Rust toolchain comes from mise (`rust = "latest"`).

## Testing Decisions

- Good tests check external behavior only: given input, they check what the user sees or what is persisted, never internal structure.
- **Seam 1, Timesheet domain** (test-first, red-green-refactor): balances, actual expected working time, "Work until", parts of day, consistency checks, warnings, sorting, time and date parsing, tested through the domain module's public interface.
- **Seam 2, Application** (tests written alongside the code): feed key sequences into the application state with a fake clock, fake clipboard and in-memory file system, render to ratatui's TestBackend, and check the rendered buffer, the persisted YAML and clipboard contents.
- A small set of tests uses a real temporary directory for the atomic write, symlinks and permissions.
- Terminal setup (Kitty keyboard protocol detection) and emoji rendering can't be checked with TestBackend; they are checked by hand in Ghostty.
- A sample timesheet file with realistic weeks serves demos and tests.

## Out of Scope

- Moco synchronization (later milestone). Project names stay free text and will be mapped to Moco then.
- A managed project list.
- Screens other than Tracking (the function-key bar already reserves F2 Track for more screens).
- Weekend days, periods crossing midnight, configurable regular expected working time, a configurable 8h target for "Work until".
- Auto-save, mouse support.
- Terminals without the Kitty keyboard protocol (ADR-0003).

## Further Notes

- **Emoji prototype (done)**: ratatui aligns 🏖️/⚠️ on its own; only the background needed fixing (ADR-0001). Prototype code: branch `prototype/emoji-alignment` (commit 383c813).
- **Second design round (2026-11-20)**: reshaped panels, keys, warnings, period types (free, part of day), the bottom line, notifications, the Help window, the command line, the clock and the schema version. This spec describes the result; tickets 12–25 implement it, and ticket 09 the project autocomplete.
