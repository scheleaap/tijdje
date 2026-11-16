# Feedback and open issues

Running list of user feedback and known open points that aren't covered by a ticket's acceptance criteria yet. Turn items into tickets (or fold them into one) when they're picked up.

## Open

- **Tab for switching panels** (user idea, 2026-11-15, after ticket 03): with ←/→ moving between fields, Tab could switch panels instead.
- **Drop panel focus** (user idea, 2026-11-16, after ticket 04): drop the concept of panel focus.
- **Clock port** (review of ticket 01): today is fixed at startup, so "today" doesn't change while the app runs past midnight. (The file-system port was added in ticket 05.)
- **Other terminals unverified** (ADR-0001): emoji rendering is only checked in Ptyxis. Konsole, kitty, foot and ghostty are still to be checked.
- **Total balance performance** (review of ticket 02): the total balance is recalculated from the first recorded day for every visible row. Fine for now; may need caching after years of data.
- **Repository history**: commit 73cad6f still mentions the employer in the README, and the prototype branch commit doesn't follow Conventional Commits.

## Done

- **Notes keeps its text when typing**: done after ticket 11. Typing in Notes adds to the text; Enter edits any field with its text; Alt+Enter/Alt+Del add/delete periods; Enter toggles checkboxes.
- **←/→ between fields**: done after ticket 11 (stops at the period's edges).
- **Shift+arrows change the day**: done after ticket 11, with Alt instead of Shift (Ptyxis uses Shift+PgUp/PgDn for scrollback); Alt+↑/↓/PgUp/PgDn replace Ctrl+↑/↓.
- **Cursor shows up in the left panel while editing**: fixed after ticket 11 (the wide-symbol repaint saves and restores the cursor).
- **Text hard to read**: done in ticket 11 (Midnight Commander palette).
- **Year in the year separator**: done in ticket 11.
- **Input field background**: done in ticket 11 (light gray fields, cyan when focused).
- **Selected day colors**: done in ticket 11 (inactive panel: cyan text, no bar).
- **Empty row between periods**: done in ticket 11.
- **Warning for 0h00m periods**: done in ticket 11.
- **Implied break as its own row**: done in ticket 11.
- **Period warning left of From**: done in ticket 11.
- **Checkboxes right of Project**: done in ticket 11.
- **Yellow selection color**: decided against in ticket 11: yellow is used for column headers.
- **Year in the left panel title**: done in ticket 11 (`Days: 2026`).
- **No padding in the right panel title**: done in ticket 07 (popups too).
- **To before From fails the check**: done in ticket 07.
- **No new period after an open last period**: done after ticket 04.
- **No new period after an empty last period**: done in ticket 04.
- **Ctrl+Del** reaches the application in Ptyxis (checked by the user after ticket 04).
- **Negative durations** (review of ticket 01): fixed in ticket 02.
