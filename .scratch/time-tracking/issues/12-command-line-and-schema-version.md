# 12: Command line options and schema version

**What to build:** `tijdje --help` and `--version` work, unknown options are refused, and the YAML file carries a schema version the application checks.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `--help` prints the usage `tijdje [PATH]`, including the default path, and exits with 0
- [ ] `--version` prints the version and exits with 0
- [ ] Any other option prints an error and exits with a non-zero code; a single non-option argument is still the timesheet path
- [ ] Saving writes `schema_version: 1` at the top of the file
- [ ] A file without `schema_version` is read as version 1; any other version than 1 makes the application refuse to start with a clear message, leaving the file untouched

Spec: `.scratch/time-tracking/spec.md`
