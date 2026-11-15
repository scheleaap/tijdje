# 01: Walking skeleton: day list from file

**What to build:** The application starts in the Norton Commander look, loads the timesheet file and shows the Tracking screen. The left panel lists days with their worked time. The right panel is an empty frame for now.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Rust crate set up (ratatui + crossterm, serde + serde_yaml_ng, jiff); builds with the toolchain from mise
- [ ] Timesheet file is read from `~/Documents/timesheet.yaml` by default, or from the path given as the single command-line argument
- [ ] YAML schema per spec: `days` keyed by `YYYY-MM-DD`, `regular_expected_working_time_in_minutes`, periods with `from`/`to` as `HH:MM`, `project`, `notes`, `type`; `null` and `""` are both read as empty
- [ ] A YAML file that can't be parsed makes the application exit with a clear error and leaves the file untouched
- [ ] A sample timesheet file with realistic weeks (vacation, sick and holiday periods, an open period, an empty day) is added to the repo for demos and tests
- [ ] Left panel lists only Monday–Friday with Dutch dates (`ma 12 nov`; `ma di wo do vr`; `jan feb mrt apr mei jun jul aug sep okt nov dec`) and worked time (`1h55m`) per day
- [ ] Worked time is the total duration of the day's work periods; open periods don't count (domain built test-first)
- [ ] Single-line separator between Friday and Monday; double-line separator between days in different years (also over a weekend)
- [ ] List scrolls endlessly into the past and future; ↑/↓ select the previous/next day, PgUp/PgDn the previous/next week; exactly one day is always selected
- [ ] On startup today is selected; on a weekend the previous Friday
- [ ] Right panel is an empty bordered frame
- [ ] Bottom line shows the function-key bar; F1 shows a help screen with all shortcuts, Esc or F2 returns to Tracking
- [ ] Ctrl+Q quits
- [ ] Classic Norton Commander palette: blue background, cyan selection bar
- [ ] Below 100×24 a "terminal too small" message is shown instead of the UI
- [ ] Application-level tests feed key sequences with a fake clock and in-memory file system and check the rendered TestBackend buffer

Spec: `.scratch/time-tracking/spec.md`
