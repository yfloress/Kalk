//! Data persistence using JSON and XDG directories.

use crate::model::Course;
use color_eyre::eyre::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

/// Get the data file path using XDG directories.
fn data_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "kalk").map(|dirs| dirs.data_dir().join("data.json"))
}

/// Load courses from disk.
/// Returns an empty vector if the file doesn't exist or can't be parsed.
pub fn load_data() -> Result<Vec<Course>> {
    let Some(path) = data_path() else {
        return Ok(Vec::new());
    };

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path).context("Failed to read data file")?;

    let courses: Vec<Course> = serde_json::from_str(&content).context("Failed to parse data")?;

    Ok(courses)
}

/// Save courses to disk.
pub fn save_data(courses: &[Course]) -> Result<()> {
    let Some(path) = data_path() else {
        return Ok(());
    };

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create data directory")?;
    }

    let content = serde_json::to_string_pretty(courses).context("Failed to serialize data")?;

    fs::write(&path, content).context("Failed to write data file")?;

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
}
