# 09: Project autocomplete

**What to build:** While typing a project, the best match from earlier projects is suggested.

**Blocked by:** 16

**Status:** done

- [x] Projects are free text
- [ ] While editing Project, the best match from projects used earlier is shown as dimmed ghost text
- [x] → at the end of the input accepts the suggestion while editing; when not editing, → moves focus

The ghost-text criterion stays unticked until the dimmed look is checked by hand in Ghostty; tests only check that the ghost text has the DIM modifier.

Spec: `.scratch/time-tracking/spec.md`
