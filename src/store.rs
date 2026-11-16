//! Loads and saves the timesheet YAML file.

use std::{
    collections::BTreeMap,
    fmt,
    fs::{self, File},
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, Write},
    path::{Path, PathBuf},
};

use jiff::civil::Date;
use serde::{Deserialize, Deserializer};

use crate::domain::{Day, Minutes, Period, PeriodType, TimeOfDay, Timesheet};

/// File access, injected so tests can use an in-memory file system.
pub trait FileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    /// Writes `contents` to `path` and flushes it to disk.
    fn write(&self, path: &Path, contents: &str) -> io::Result<()>;
    /// Copies `from` to `to` and flushes the copy to disk.
    fn copy(&self, from: &Path, to: &Path) -> io::Result<()>;
    /// Replaces `to` with `from` in one step, keeping `to`'s permissions.
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn remove(&self, path: &Path) -> io::Result<()>;
    /// The file `path` refers to, following symlinks; `path` itself if it
    /// isn't a symlink or doesn't exist.
    fn resolve(&self, path: &Path) -> PathBuf;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn write(&self, path: &Path, contents: &str) -> io::Result<()> {
        let mut file = File::create(path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    }

    fn copy(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::copy(from, to)?;
        File::open(to)?.sync_all()
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        if let Ok(metadata) = fs::metadata(to) {
            fs::set_permissions(from, metadata.permissions())?;
        }
        fs::rename(from, to)?;
        // Flush the folder too, so the rename itself survives a power loss.
        let folder = match to.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent,
            _ => Path::new("."),
        };
        File::open(folder)?.sync_all()
    }

    fn remove(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn resolve(&self, path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_owned())
    }
}

/// The file's contents, or `None` if it doesn't exist.
fn read_if_exists(fs: &dyn FileSystem, path: &Path) -> io::Result<Option<String>> {
    match fs.read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

/// What the file looked like when it was last loaded or saved, to detect
/// changes made outside the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint(Option<u64>);

impl Fingerprint {
    /// The fingerprint of a file's contents, or of a missing file (`None`).
    fn of(text: Option<&str>) -> Self {
        Self(text.map(|text| {
            let mut hasher = DefaultHasher::new();
            text.hash(&mut hasher);
            hasher.finish()
        }))
    }
}

#[derive(Debug)]
pub enum LoadError {
    Io(io::Error),
    Parse(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::Io(e) => write!(f, "cannot read the timesheet file: {e}"),
            LoadError::Parse(e) => write!(f, "cannot parse the timesheet file: {e}"),
        }
    }
}

#[derive(Debug)]
pub enum SaveError {
    /// The file changed on disk since it was loaded or last saved.
    ChangedOnDisk,
    Io(io::Error),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SaveError::ChangedOnDisk => write!(f, "the file changed on disk"),
            SaveError::Io(e) => write!(f, "cannot save the timesheet file: {e}"),
        }
    }
}

/// Whether saving may replace a file that changed on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overwrite {
    IfUnchanged,
    Always,
}

/// Whether saving keeps the file it replaces as `<file>.bak`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backup {
    Create,
    Skip,
}

/// Loads the timesheet at `path`. A missing or empty file is an empty timesheet.
pub fn load(fs: &dyn FileSystem, path: &Path) -> Result<(Timesheet, Fingerprint), LoadError> {
    let text = read_if_exists(fs, path).map_err(LoadError::Io)?;
    let fingerprint = Fingerprint::of(text.as_deref());
    let Some(text) = text.filter(|text| !text.trim().is_empty()) else {
        return Ok((Timesheet::default(), fingerprint));
    };
    let file: FileModel =
        serde_yaml_ng::from_str(&text).map_err(|e| LoadError::Parse(e.to_string()))?;
    Ok((
        file.into_timesheet().map_err(LoadError::Parse)?,
        fingerprint,
    ))
}

/// Saves the timesheet to `path`. If `path` is a symlink, the file it points
/// to is replaced and the symlink stays; the temporary file and the backup
/// go next to that file.
///
/// The YAML is written to a temporary file, flushed to disk and renamed over
/// the old file, so a crash never leaves a half-written file. With
/// `Overwrite::IfUnchanged`, saving is refused when the file changed on disk
/// since `expected` was taken.
pub fn save(
    fs: &dyn FileSystem,
    path: &Path,
    timesheet: &Timesheet,
    expected: &Fingerprint,
    overwrite: Overwrite,
    backup: Backup,
) -> Result<Fingerprint, SaveError> {
    let target = fs.resolve(path);
    let current = read_if_exists(fs, &target).map_err(SaveError::Io)?;
    if overwrite == Overwrite::IfUnchanged && Fingerprint::of(current.as_deref()) != *expected {
        return Err(SaveError::ChangedOnDisk);
    }
    let text = to_yaml(timesheet);
    let temporary = with_suffix(&target, ".tmp");
    let replace = || {
        fs.write(&temporary, &text)?;
        if backup == Backup::Create && current.is_some() {
            fs.copy(&target, &with_suffix(&target, ".bak"))?;
        }
        fs.rename(&temporary, &target)
    };
    if let Err(e) = replace() {
        let _ = fs.remove(&temporary);
        return Err(SaveError::Io(e));
    }
    Ok(Fingerprint::of(Some(&text)))
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Writes the timesheet as YAML. Dates, times and text are always
/// double-quoted, so YAML 1.1 readers don't take `08:45` for a number or
/// `2026-11-02` for a timestamp. Days without periods are not written, and
/// every written day gets its regular expected working time, so a new day
/// stores the default.
fn to_yaml(timesheet: &Timesheet) -> String {
    let days: Vec<_> = timesheet
        .days
        .iter()
        .filter(|(_, day)| !day.is_empty())
        .collect();
    if days.is_empty() {
        return "days: {}\n".to_owned();
    }
    let mut out = String::from("days:\n");
    for (date, day) in days {
        out += &format!(
            "  \"{date}\":\n    regular_expected_working_time_in_minutes: {}\n    periods:\n",
            day.regular_expected_working_time().0
        );
        for period in &day.periods {
            out += &format!(
                "      - from: {}\n        to: {}\n        project: {}\n        notes: {}\n        type: {}\n",
                yaml_time(period.from),
                yaml_time(period.to),
                yaml_text(period.project.as_deref()),
                yaml_text(period.notes.as_deref()),
                period_type_name(period.period_type),
            );
        }
    }
    out
}

fn yaml_time(time: Option<TimeOfDay>) -> String {
    time.map_or("null".to_owned(), |t| {
        format!("\"{}\"", format_stored_time(t))
    })
}

/// A double-quoted YAML string, or `null`.
fn yaml_text(text: Option<&str>) -> String {
    let Some(text) = text else {
        return "null".to_owned();
    };
    let mut quoted = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => quoted += "\\\"",
            '\\' => quoted += "\\\\",
            '\n' => quoted += "\\n",
            '\r' => quoted += "\\r",
            '\t' => quoted += "\\t",
            c if c.is_control() => quoted += &format!("\\u{:04x}", u32::from(c)),
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// The stored name of each period type.
const PERIOD_TYPE_NAMES: [(PeriodType, &str); 5] = [
    (PeriodType::Work, "work"),
    (PeriodType::Break, "break"),
    (PeriodType::Vacation, "vacation"),
    (PeriodType::Sick, "sick"),
    (PeriodType::Holiday, "holiday"),
];

fn period_type_name(period_type: PeriodType) -> &'static str {
    PERIOD_TYPE_NAMES
        .iter()
        .find(|(t, _)| *t == period_type)
        .map(|(_, name)| *name)
        .expect("every period type has a name")
}

fn period_type_from_name(name: &str) -> Option<PeriodType> {
    PERIOD_TYPE_NAMES
        .iter()
        .find(|(_, n)| *n == name)
        .map(|(t, _)| *t)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FileModel {
    #[serde(default)]
    days: BTreeMap<Date, DayModel>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DayModel {
    regular_expected_working_time_in_minutes: i32,
    #[serde(default)]
    periods: Vec<PeriodModel>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PeriodModel {
    #[serde(default, deserialize_with = "empty_as_none")]
    from: Option<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    to: Option<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    project: Option<String>,
    #[serde(default, deserialize_with = "empty_as_none")]
    notes: Option<String>,
    #[serde(rename = "type")]
    period_type: String,
}

fn empty_as_none<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<String>::deserialize(d)?.filter(|s| !s.is_empty()))
}

/// Formats a time in the stored format `HH:MM`.
fn format_stored_time(time: TimeOfDay) -> String {
    let minutes = time.minutes_since_midnight();
    format!("{:02}:{:02}", minutes / 60, minutes % 60)
}

/// Parses the stored time format `HH:MM`.
fn parse_stored_time(s: &str) -> Option<TimeOfDay> {
    let (h, m) = s.split_once(':')?;
    if h.len() != 2 || m.len() != 2 {
        return None;
    }
    TimeOfDay::new(h.parse().ok()?, m.parse().ok()?)
}

impl FileModel {
    fn into_timesheet(self) -> Result<Timesheet, String> {
        let mut days = BTreeMap::new();
        for (date, day) in self.days {
            let periods = day
                .periods
                .into_iter()
                .map(|p| p.into_period().map_err(|e| format!("{date}: {e}")))
                .collect::<Result<_, _>>()?;
            days.insert(
                date,
                Day {
                    regular_expected_working_time: Some(Minutes(
                        day.regular_expected_working_time_in_minutes,
                    )),
                    periods,
                },
            );
        }
        Ok(Timesheet { days })
    }
}

impl PeriodModel {
    fn into_period(self) -> Result<Period, String> {
        let time = |s: Option<String>| match s {
            None => Ok(None),
            Some(s) => parse_stored_time(&s)
                .map(Some)
                .ok_or_else(|| format!("invalid time {s:?}, expected HH:MM")),
        };
        Ok(Period {
            from: time(self.from)?,
            to: time(self.to)?,
            project: self.project,
            notes: self.notes,
            period_type: period_type_from_name(&self.period_type)
                .ok_or_else(|| format!("unknown period type {:?}", self.period_type))?,
        })
    }
}
