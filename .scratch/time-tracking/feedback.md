# Feedback and open issues

Running list of user feedback and known open points that aren't covered by a ticket's acceptance criteria yet. Turn items into tickets (or fold them into one) when they're picked up.

## Open

- **Left panel border breaks when scrolling** (user bug report, 2026-11-20, after tickets 14/17): after scrolling the day list, the left panel's right border is missing on rows that scrolled in (screenshot from the user). Likely related to wide emoji and partial redraws (ratatui assuming 2-cell emoji while the terminal advances differently, or a leftover right half of a wide symbol, or the ADR-0001 background repaint). Unknown yet: which terminal, and whether a resize (full redraw) fixes it. To be diagnosed by reproducing it first.
- **Other terminals unverified** (ADR-0001, ADR-0003): emoji rendering is only checked in Ptyxis, which ADR-0003 drops. Ghostty (now the main terminal), kitty, foot and Alacritty are still to be checked.
- **Total balance performance** (review of ticket 02): the total balance is recalculated from the first recorded day for every visible row. Fine for now; may need caching after years of data.

## Done

- **Drop panel focus**: planned in ticket 15.
- **No "0h" for durations under an hour**: planned in ticket 22.
- **Empty field on a day without periods**: planned in ticket 15.
- **Alt+1/2/3 fill a day fraction**: superseded by part of day (ticket 21).
- **Remaining time**: superseded by the new bottom line (ticket 22).
- **`--help` argument**: planned in ticket 12.
- **Clock port**: planned in ticket 13.
- **Notes keeps its text when typing**: done after ticket 11. Typing in Notes adds to the text; Enter edits any field with its text; Alt+Enter/Alt+Del add/delete periods; Enter toggles checkboxes. Superseded by the second design round (see the spec).
- **←/→ between fields**: done after ticket 11 (stops at the period's edges).
- **Shift+arrows change the day**: done after ticket 11, with Alt instead of Shift (Ptyxis uses Shift+PgUp/PgDn for scrollback); Alt+↑/↓/PgUp/PgDn replace Ctrl+↑/↓. Superseded by the second design round (see the spec).
- **Cursor shows up in the left panel while editing**: fixed after ticket 11 (the wide-symbol repaint saves and restores the cursor).
- **Text hard to read**: done in ticket 11 (Midnight Commander palette).
- **Year in the year separator**: done in ticket 11.
- **Input field background**: done in ticket 11 (light gray fields, cyan when focused).
- **Selected day colors**: done in ticket 11 (inactive panel: cyan text, no bar). Superseded by the second design round (see the spec).
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

## Rejected

- **Tab for switching panels** (user idea, 2026-11-15, after ticket 03): with ←/→ moving between fields, Tab could switch panels instead.
- **Repository history**: commit 73cad6f still mentions the employer in the README, and the prototype branch commit doesn't follow Conventional Commits.
