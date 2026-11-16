# 05: Save and quit safely

**What to build:** Changes are saved to the timesheet file safely, and quitting never loses work by accident.

**Blocked by:** 04

**Status:** done

- [x] Ctrl+S saves (consistency checks come in ticket 07)
- [x] Writes are atomic (temp file + rename) and keep the previous file as `timesheet.yaml.bak`
- [x] Empty To, project and notes are written as `null`; `type` is always written; days without periods are not written
- [x] `regular_expected_working_time_in_minutes` is written when a day gets its first period; existing values are kept
- [x] If the file changed on disk since loading or last saving, saving asks "File changed on disk (Overwrite/Cancel)"
- [x] Ctrl+Q with unsaved changes asks "Save before quitting? (Save/Don't save/Cancel)"
- [x] Ctrl+S and Ctrl+Q are ignored while editing (A)
- [x] A small set of tests uses a real temporary directory for the atomic write and `.bak`

Spec: `.scratch/time-tracking/spec.md`
