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

//! Data persistence using JSON and XDG directories.

use crate::i18n::Language;
use crate::model::{Course, CourseTemplate, Semester};
use color_eyre::eyre::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};
use uuid::Uuid;

/// Application configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub language: Language,
    #[serde(default)]
    pub use_nerd_fonts: bool,
    /// Whether the last session ended on the Home screen. Defaults to `true`
    /// only for a brand-new install, so a first run lands on the dashboard.
    #[serde(default)]
    pub start_on_home: bool,
    /// Semester open when the last session ended.
    #[serde(default)]
    pub last_semester: Option<Uuid>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::English,
            use_nerd_fonts: true,
            start_on_home: true,
            last_semester: None,
        }
    }
}

/// Get the data file path using XDG directories.
fn data_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "kalk").map(|dirs| dirs.data_dir().join("data.json"))
}

/// Get the user templates file path using XDG directories.
fn user_templates_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "kalk").map(|dirs| dirs.data_dir().join("user_templates.json"))
}

/// Get the config file path using XDG directories.
fn config_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "kalk").map(|dirs| dirs.data_dir().join("config.json"))
}

/// Current on-disk schema version for `data.json`.
const DATA_SCHEMA_VERSION: u32 = 2;

/// On-disk shape of `data.json` from v2 onwards.
#[derive(Debug, Serialize, Deserialize)]
struct DataFile {
    schema_version: u32,
    semesters: Vec<Semester>,
}

/// Either shape `data.json` may have on disk. Untagged matching is
/// unambiguous: v1 was a bare array, v2 is an object.
#[derive(Deserialize)]
#[serde(untagged)]
enum StoredData {
    Versioned(DataFile),
    Legacy(Vec<Course>),
}

/// Load semesters from disk, migrating the v1 format if that is what is there.
/// `legacy_semester_name` (already translated) names the semester that v1
/// courses get wrapped into.
pub fn load_data(legacy_semester_name: &str) -> Result<Vec<Semester>> {
    let Some(path) = data_path() else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path).context("Failed to open data file")?;
    let reader = BufReader::new(file);

    let stored: StoredData = serde_json::from_reader(reader).context("Failed to parse data")?;

    if matches!(stored, StoredData::Legacy(_)) {
        back_up_legacy_data(&path);
    }

    Ok(migrate(stored, legacy_semester_name))
}

/// Bring either on-disk shape to the current one. Pure, so it is testable
/// without touching the filesystem.
fn migrate(stored: StoredData, legacy_semester_name: &str) -> Vec<Semester> {
    match stored {
        StoredData::Versioned(data) => data.semesters,
        StoredData::Legacy(courses) if courses.is_empty() => Vec::new(),
        StoredData::Legacy(courses) => {
            let mut semester = Semester::new(legacy_semester_name.to_string(), 0);
            semester.courses = courses;
            vec![semester]
        }
    }
}

/// Copy the v1 file aside before v2 overwrites it. Never overwrites an
/// existing backup, so a later bad state cannot clobber a good one.
fn back_up_legacy_data(path: &PathBuf) {
    let backup = path.with_extension("v1.bak");
    if !backup.exists() {
        let _ = fs::copy(path, &backup);
    }
}

/// Save semesters to disk.
pub fn save_data(semesters: &[Semester]) -> Result<()> {
    let Some(path) = data_path() else {
        return Ok(());
    };

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create data directory")?;
    }

    let tmp_path = path.with_extension("tmp");
    let file = File::create(&tmp_path).context("Failed to create temp data file")?;
    let mut writer = BufWriter::new(file);

    let data = DataFile {
        schema_version: DATA_SCHEMA_VERSION,
        semesters: semesters.to_vec(),
    };
    serde_json::to_writer_pretty(&mut writer, &data).context("Failed to serialize data")?;
    writer.flush().context("Failed to flush data to disk")?;
    drop(writer);

    if let Err(err) = fs::rename(&tmp_path, &path) {
        if path.exists() {
            fs::remove_file(&path).context("Failed to remove existing data file")?;
            if let Err(rename_err) = fs::rename(&tmp_path, &path) {
                let _ = fs::remove_file(&tmp_path);
                return Err(rename_err).context("Failed to write data file");
            }
        } else {
            let _ = fs::remove_file(&tmp_path);
            return Err(err).context("Failed to write data file");
        }
    }

    Ok(())
}

/// Load user-created templates from disk.
/// Returns an empty vector if the file doesn't exist or can't be parsed.
pub fn load_user_templates() -> Result<Vec<CourseTemplate>> {
    let Some(path) = user_templates_path() else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let file = File::open(&path).context("Failed to open user templates file")?;
    let reader = BufReader::new(file);

    let templates: Vec<CourseTemplate> =
        serde_json::from_reader(reader).context("Failed to parse user templates")?;

    Ok(templates)
}

/// Save user-created templates to disk.
pub fn save_user_templates(templates: &[CourseTemplate]) -> Result<()> {
    let Some(path) = user_templates_path() else {
        return Ok(());
    };

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create data directory")?;
    }

    let tmp_path = path.with_extension("tmp");
    let file = File::create(&tmp_path).context("Failed to create temp user templates file")?;
    let mut writer = BufWriter::new(file);

    serde_json::to_writer_pretty(&mut writer, templates)
        .context("Failed to serialize user templates")?;
    writer
        .flush()
        .context("Failed to flush user templates to disk")?;
    drop(writer);

    if let Err(err) = fs::rename(&tmp_path, &path) {
        if path.exists() {
            fs::remove_file(&path).context("Failed to remove existing user templates file")?;
            if let Err(rename_err) = fs::rename(&tmp_path, &path) {
                let _ = fs::remove_file(&tmp_path);
                return Err(rename_err).context("Failed to write user templates file");
            }
        } else {
            let _ = fs::remove_file(&tmp_path);
            return Err(err).context("Failed to write user templates file");
        }
    }

    Ok(())
}

/// Load configuration from disk.
/// Returns default config if the file doesn't exist or can't be parsed.
/// The optional string contains a warning message to surface to the user.
pub fn load_config() -> (Config, Option<String>) {
    let Some(path) = config_path() else {
        return (Config::default(), None);
    };

    if !path.exists() {
        return (Config::default(), None);
    }

    let file = match File::open(&path) {
        Ok(file) => file,
        Err(_) => {
            return (Config::default(), Some("config_load_warning".to_string()));
        }
    };

    let reader = BufReader::new(file);
    match serde_json::from_reader(reader) {
        Ok(cfg) => (cfg, None),
        Err(_) => (Config::default(), Some("config_load_warning".to_string())),
    }
}

/// Save configuration to disk.
pub fn save_config(config: &Config) -> Result<()> {
    let Some(path) = config_path() else {
        return Ok(());
    };

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    let tmp_path = path.with_extension("tmp");
    let file = File::create(&tmp_path).context("Failed to create temp config file")?;
    let mut writer = BufWriter::new(file);

    serde_json::to_writer_pretty(&mut writer, config).context("Failed to serialize config")?;
    writer.flush().context("Failed to flush config to disk")?;
    drop(writer);

    if let Err(err) = fs::rename(&tmp_path, &path) {
        if path.exists() {
            fs::remove_file(&path).context("Failed to remove existing config file")?;
            if let Err(rename_err) = fs::rename(&tmp_path, &path) {
                let _ = fs::remove_file(&tmp_path);
                return Err(rename_err).context("Failed to write config file");
            }
        } else {
            let _ = fs::remove_file(&tmp_path);
            return Err(err).context("Failed to write config file");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_path_exists() {
        // Just verify we can get a path
        let path = data_path();
        assert!(path.is_some());
    }

    #[test]
    fn test_user_templates_path_exists() {
        let path = user_templates_path();
        assert!(path.is_some());
    }

    #[test]
    fn test_config_path_exists() {
        let path = config_path();
        assert!(path.is_some());
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.language, Language::English);
        assert!(config.use_nerd_fonts);
    }

    // =========================================================================
    // data.json migration (v1 -> v2)
    // =========================================================================

    const V1_JSON: &str = r#"[
        {"id":"00000000-0000-0000-0000-000000000001","name":"Calculus",
         "passing_grade":55.0,"categories":[]}
    ]"#;

    #[test]
    fn test_v1_array_is_wrapped_into_one_semester() {
        let stored: StoredData = serde_json::from_str(V1_JSON).unwrap();
        let semesters = migrate(stored, "Current");

        assert_eq!(semesters.len(), 1);
        assert_eq!(semesters[0].name, "Current");
        assert_eq!(semesters[0].order, 0);
        assert_eq!(semesters[0].courses.len(), 1);
        assert_eq!(semesters[0].courses[0].name, "Calculus");
    }

    #[test]
    fn test_empty_v1_array_produces_no_semester() {
        let stored: StoredData = serde_json::from_str("[]").unwrap();
        assert!(migrate(stored, "Current").is_empty());
    }

    #[test]
    fn test_v2_object_is_read_as_is() {
        let json = r#"{"schema_version":2,"semesters":[
            {"id":"00000000-0000-0000-0000-000000000002","name":"2025-2",
             "order":3,"courses":[]}
        ]}"#;
        let stored: StoredData = serde_json::from_str(json).unwrap();
        let semesters = migrate(stored, "Current");

        assert_eq!(semesters.len(), 1);
        assert_eq!(semesters[0].name, "2025-2");
        assert_eq!(semesters[0].order, 3);
    }

    #[test]
    fn test_v2_round_trip_reloads_identically() {
        let mut semester = Semester::new("2026-1".to_string(), 0);
        semester
            .courses
            .push(Course::new("Physics".to_string(), 55.0));
        let data = DataFile {
            schema_version: DATA_SCHEMA_VERSION,
            semesters: vec![semester],
        };

        let json = serde_json::to_string(&data).unwrap();
        let stored: StoredData = serde_json::from_str(&json).unwrap();
        let semesters = migrate(stored, "Current");

        assert_eq!(semesters.len(), 1);
        assert_eq!(semesters[0].courses[0].name, "Physics");
    }
}
