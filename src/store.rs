//! Loads the timesheet YAML file.

use std::{collections::BTreeMap, fmt, io, path::Path};

use jiff::civil::Date;
use serde::{Deserialize, Deserializer};

use crate::domain::{Day, Minutes, Period, PeriodType, TimeOfDay, Timesheet};

/// File access, injected so tests can use an in-memory file system.
pub trait FileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
}

pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        std::fs::read_to_string(path)
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

/// Loads the timesheet at `path`. A missing or empty file is an empty timesheet.
pub fn load(fs: &dyn FileSystem, path: &Path) -> Result<Timesheet, LoadError> {
    let text = match fs.read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Timesheet::default()),
        Err(e) => return Err(LoadError::Io(e)),
    };
    if text.trim().is_empty() {
        return Ok(Timesheet::default());
    }
    let file: FileModel =
        serde_yaml_ng::from_str(&text).map_err(|e| LoadError::Parse(e.to_string()))?;
    file.into_timesheet().map_err(LoadError::Parse)
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
    regular_expected_working_time_in_minutes: Option<i32>,
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
    period_type: PeriodTypeModel,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum PeriodTypeModel {
    Work,
    Break,
    Vacation,
    Sick,
    Holiday,
}

fn empty_as_none<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Ok(Option::<String>::deserialize(d)?.filter(|s| !s.is_empty()))
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
                    regular_expected_working_time: day
                        .regular_expected_working_time_in_minutes
                        .map(Minutes),
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
            period_type: match self.period_type {
                PeriodTypeModel::Work => PeriodType::Work,
                PeriodTypeModel::Break => PeriodType::Break,
                PeriodTypeModel::Vacation => PeriodType::Vacation,
                PeriodTypeModel::Sick => PeriodType::Sick,
                PeriodTypeModel::Holiday => PeriodType::Holiday,
            },
        })
    }
}
