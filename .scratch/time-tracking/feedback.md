# Feedback and open issues

Running list of user feedback and known open points that aren't covered by a ticket's acceptance criteria yet. Turn items into tickets (or fold them into one) when they're picked up.

## Open

- **Text hard to read** (user feedback, 2026-11-15, after ticket 01): cyan text on blue has low contrast in the left panel. Possible directions: light cyan or white text, as in the original Norton Commander palette. User liked the line style and colors of the emoji alignment prototype (branch `prototype/emoji-alignment`): single-line borders and white text on blue.
- **Year in the year separator** (user feedback, 2026-11-15, after ticket 02): show the year centered in the double-line separator, e.g. `║═════════════════ 2027 ═════════════════════║`.
- **←/→ between fields** (user feedback, 2026-11-15, after ticket 03): ←/→ should also move between fields in the right panel. Idea to think about later: that would free up Tab for switching panels.
- **Input field background** (user feedback, 2026-11-15, after ticket 03): the darker blue background of input fields doesn't match the rest of the color scheme.
- **Selected day colors** (user feedback, 2026-11-15, after ticket 03): the gray bar of the selected day in an inactive left panel draws more attention than the cyan bar of the active panel.
- **Empty row between periods** (user feedback, 2026-11-15, after ticket 03): show an empty row between two periods in the right panel.
- **Warning for 0h00m periods** (user feedback, 2026-11-16, after ticket 04): a period with a duration of 0h00m should also get the ⚠️ indicator.
- **To before From fails the check** (user feedback, 2026-11-16, after ticket 04): already covered by ticket 07 ("From before To"). Until then, such a period shows a negative duration.
- **Drop panel focus** (user idea, 2026-11-16, after ticket 04): drop the concept of panel focus.
- **Shift+arrows change the day** (user idea, 2026-11-16, after ticket 04): Shift+↑/↓ and Shift+PgUp/PgDn change the selected day.
- **No padding in the right panel title** (user feedback, 2026-11-16, after ticket 04): the day number in the right panel title should not be padded (`ma 2 nov`); in the left panel list it stays padded.
- **Implied break as its own row** (user feedback, 2026-11-16, after ticket 04): render an implied break as a separate row instead of an indicator next to a period. No input fields: From and To shown as plain text, the Break checkbox checked, and the checkboxes don't take input.
- **Period warning left of From** (user feedback, 2026-11-16, after ticket 04): show the period's ⚠️ indicator to the left of From.
- **Checkboxes right of Project** (user feedback, 2026-11-16, after ticket 04): position the checkboxes directly to the right of the Project field.
- **Yellow selection color** (user idea, 2026-11-16, after ticket 04): consider yellow as the selection color.
- **Year in the left panel title** (user feedback, 2026-11-16, after ticket 04): the left panel title should also show the year.
- **Clock port** (review of ticket 01): today is fixed at startup, so "today" doesn't change while the app runs past midnight. (The file-system port was added in ticket 05.)
- **Other terminals unverified** (ADR-0001): emoji rendering is only checked in Ptyxis. Konsole, kitty, foot and ghostty are still to be checked.
- **Total balance performance** (review of ticket 02): the total balance is recalculated from the first recorded day for every visible row. Fine for now; may need caching after years of data.
- **Repository history**: commit 73cad6f still mentions the employer in the README, and the prototype branch commit doesn't follow Conventional Commits.

## Done

- **No new period after an open last period**: done after ticket 04.
- **No new period after an empty last period**: done in ticket 04.
- **Ctrl+Del** reaches the application in Ptyxis (checked by the user after ticket 04).
- **Negative durations** (review of ticket 01): fixed in ticket 02.
