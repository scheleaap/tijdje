## Agent skills

### Issue tracker

Issues live as local markdown files under `.scratch/<feature>/`. See `docs/agents/issue-tracker.md`.

### Domain docs

Single-context: one `GLOSSARY.md` and `docs/adr/` at the repo root. See `docs/agents/domain.md`.

## Workflow

### Merging a ticket

Squash-merge each ticket into one commit on `main`, keeping history linear:

- Subject: the ticket's main change as a Conventional Commit, e.g. `feat: Add the free period type`.
- Body: the main parts of the change, then a last line `Ticket: <number>`.
- Contents: everything for the ticket, including tests, review fixes, spec updates and marking the ticket done.

Ticket branches may keep small commits; they exist for the reviews.
