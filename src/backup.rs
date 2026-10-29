// Kalk — your academic dashboard in the terminal.
// Copyright (C) 2026  yfloress
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/agpl-3.0.html>.
//

//! Whole-library backups: every semester and user template in one file, so
//! moving to another computer is "export" on one and "import" on the other.

use std::fs;
use std::path::{Path, PathBuf};

use directories::UserDirs;
use serde::{Deserialize, Serialize};

use crate::i18n::Messages;
use crate::model::{CourseTemplate, Semester};
use crate::persistence;

/// Shape of the backup file. Semesters are stored exactly as in `data.json`,
/// so a change there means bumping this too.
pub const BACKUP_VERSION: u32 = 1;

/// Exports are named `kalk-backup-YYYY-MM-DD.json`; sorting by name is
/// sorting by date.
const FILE_PREFIX: &str = "kalk-backup-";
const FILE_SUFFIX: &str = ".json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    /// Marks the file as a Kalk backup and versions its shape.
    pub kalk_backup: u32,
    pub semesters: Vec<Semester>,
    #[serde(default)]
    pub user_templates: Vec<CourseTemplate>,
}

impl Backup {
    pub fn new(semesters: &[Semester], user_templates: &[CourseTemplate]) -> Self {
        Self {
            kalk_backup: BACKUP_VERSION,
            semesters: semesters.to_vec(),
            user_templates: user_templates.to_vec(),
        }
    }

    pub fn course_count(&self) -> usize {
        self.semesters.iter().map(|s| s.courses.len()).sum()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum BackupError {
    NotFound,
    /// Not JSON, or JSON that is not a Kalk backup.
    NotABackup,
    /// Written by a newer Kalk than this one.
    Newer(u32),
    /// A backup with no semesters would leave the app with nothing to show.
    Empty,
}

impl BackupError {
    pub fn user_message(&self, m: &Messages) -> String {
        match self {
            BackupError::NotFound => m.backup_err_not_found.to_string(),
            BackupError::NotABackup => m.backup_err_not_a_backup.to_string(),
            BackupError::Newer(v) => format!("{} ({} > {})", m.backup_err_newer, v, BACKUP_VERSION),
            BackupError::Empty => m.backup_err_empty.to_string(),
        }
    }
}

// =============================================================================
// Reading and writing
// =============================================================================

/// Write every semester and user template to `path`.
pub fn export(path: &Path, backup: &Backup) -> color_eyre::Result<()> {
    persistence::write_json_atomic(path, backup)
}

/// Read and validate the backup at `path`.
pub fn read(path: &Path) -> Result<Backup, BackupError> {
    let text = fs::read_to_string(path).map_err(|_| BackupError::NotFound)?;
    parse(&text)
}

/// Validate a backup's text. Pure, so it is testable without the filesystem.
fn parse(text: &str) -> Result<Backup, BackupError> {
    let backup: Backup = serde_json::from_str(text).map_err(|_| BackupError::NotABackup)?;
    if backup.kalk_backup > BACKUP_VERSION {
        return Err(BackupError::Newer(backup.kalk_backup));
    }
    if backup.semesters.is_empty() {
        return Err(BackupError::Empty);
    }
    Ok(backup)
}

// =============================================================================
// Paths
// =============================================================================

fn home_dir() -> Option<PathBuf> {
    UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf())
}

/// Where today's export goes: the home folder, where it is easy to find and
/// copy to a USB stick or a cloud folder.
pub fn default_export_path() -> Option<PathBuf> {
    let name = format!(
        "{}{}{}",
        FILE_PREFIX,
        civil_date(persistence::unix_seconds()),
        FILE_SUFFIX
    );
    home_dir().map(|home| home.join(name))
}

/// Where a shared course goes: `~/kalk-course-<name>.json`, with anything
/// that is awkward in a file name turned into dashes.
pub fn course_share_path(course_name: &str) -> Option<PathBuf> {
    home_dir().map(|home| home.join(format!("kalk-course-{}.json", file_slug(course_name))))
}

fn file_slug(name: &str) -> String {
    let dashed: String = name
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();
    let slug = dashed
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if slug.is_empty() {
        "course".to_string()
    } else {
        slug
    }
}

/// The newest export in the home folder, to prefill the import path.
pub fn latest_in_home() -> Option<PathBuf> {
    let home = home_dir()?;
    fs::read_dir(&home)
        .ok()?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.starts_with(FILE_PREFIX) && name.ends_with(FILE_SUFFIX))
        .max()
        .map(|name| home.join(name))
}

/// Expand a leading `~` the way a shell would, since that is how people type
/// paths. Surrounding quotes, which a file manager adds when a file is
/// dragged into the terminal, are dropped too.
pub fn expand_path(input: &str) -> PathBuf {
    let trimmed = input.trim().trim_matches(|c| c == '\'' || c == '"');
    match (trimmed.strip_prefix('~'), home_dir()) {
        (Some(rest), Some(home)) if rest.is_empty() || rest.starts_with('/') => {
            home.join(rest.trim_start_matches('/'))
        }
        _ => PathBuf::from(trimmed),
    }
}

/// `path` with the home folder shown as `~`, which is shorter and is what the
/// user would type back.
pub fn display_path(path: &Path) -> String {
    match home_dir().and_then(|home| path.strip_prefix(&home).ok().map(Path::to_path_buf)) {
        Some(rest) => format!("~/{}", rest.display()),
        None => path.display().to_string(),
    }
}

/// `YYYY-MM-DD` (UTC) for a Unix timestamp, without pulling in a date crate.
/// Howard Hinnant's days-to-civil algorithm.
fn civil_date(unix_seconds: u64) -> String {
    let days = (unix_seconds / 86_400) as i64 + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{:04}-{:02}-{:02}", year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_date_matches_known_days() {
        assert_eq!(civil_date(0), "1970-01-01");
        assert_eq!(civil_date(951_782_400), "2000-02-29");
        assert_eq!(civil_date(1_793_232_000), "2026-10-29");
    }

    #[test]
    fn a_backup_round_trips() {
        let mut semester = Semester::new("2026-1".to_string(), 0);
        semester
            .courses
            .push(crate::model::Course::new("Physics".to_string(), 55.0));
        let backup = Backup::new(&[semester], &[]);

        let text = serde_json::to_string(&backup).unwrap();
        let back = parse(&text).unwrap();
        assert_eq!(back.semesters.len(), 1);
        assert_eq!(back.course_count(), 1);
    }

    #[test]
    fn data_json_is_not_mistaken_for_a_backup() {
        let data = r#"{"schema_version":2,"semesters":[]}"#;
        assert_eq!(parse(data).unwrap_err(), BackupError::NotABackup);
        assert_eq!(parse("not json").unwrap_err(), BackupError::NotABackup);
    }

    #[test]
    fn a_newer_backup_is_refused() {
        let text = r#"{"kalk_backup":99,"semesters":[]}"#;
        assert_eq!(parse(text).unwrap_err(), BackupError::Newer(99));
    }

    #[test]
    fn an_empty_backup_is_refused() {
        let text = r#"{"kalk_backup":1,"semesters":[]}"#;
        assert_eq!(parse(text).unwrap_err(), BackupError::Empty);
    }

    #[test]
    fn course_names_become_tidy_file_names() {
        assert_eq!(file_slug("Cálculo III (2026)"), "Cálculo-III-2026");
        assert_eq!(file_slug("  / "), "course");
    }

    #[test]
    fn quotes_from_a_dragged_file_are_dropped() {
        assert_eq!(
            expand_path("'/tmp/kalk-backup.json'"),
            PathBuf::from("/tmp/kalk-backup.json")
        );
    }
}
