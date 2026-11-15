# Feedback and open issues

Running list of user feedback and known open points that aren't covered by a ticket's acceptance criteria yet. Turn items into tickets (or fold them into one) when they're picked up.

## Open

- **Text hard to read** (user feedback, 2026-11-15, after ticket 01): cyan text on blue has low contrast in the left panel. Possible directions: light cyan or white text, as in the original Norton Commander palette.
- **Negative durations** (review of ticket 01): `Minutes::format_duration` produces wrong output for negative values. Fix in ticket 02, where signed balances are needed.
- **Clock and file-system ports** (review of ticket 01): today is fixed at startup, and the app has no file-system port yet. Needed for ticket 05 (saving), and for "today" changing while the app runs past midnight.
- **Other terminals unverified** (ADR-0001): emoji rendering is only checked in Ptyxis. Konsole, kitty, foot and ghostty are still to be checked.
- **Ctrl+Del** may not reach the application in every terminal. Check when implementing ticket 04.
- **Repository history**: commit 73cad6f still mentions the employer in the README, and the prototype branch commit doesn't follow Conventional Commits.

## Done
