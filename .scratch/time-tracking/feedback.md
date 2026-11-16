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
- **Clock and file-system ports** (review of ticket 01): today is fixed at startup, and the app has no file-system port yet. Needed for ticket 05 (saving), and for "today" changing while the app runs past midnight.
- **Other terminals unverified** (ADR-0001): emoji rendering is only checked in Ptyxis. Konsole, kitty, foot and ghostty are still to be checked.
- **Total balance performance** (review of ticket 02): the total balance is recalculated from the first recorded day for every visible row. Fine for now; may need caching after years of data.
- **Repository history**: commit 73cad6f still mentions the employer in the README, and the prototype branch commit doesn't follow Conventional Commits.

## Done

- **No new period after an empty last period**: done in ticket 04.
- **Ctrl+Del** reaches the application in Ptyxis (checked by the user after ticket 04).

- **Negative durations** (review of ticket 01): fixed in ticket 02.
