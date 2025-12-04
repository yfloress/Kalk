//! Data persistence using JSON and XDG directories.

use crate::model::Course;
use color_eyre::eyre::{Context, Result};
use directories::ProjectDirs;
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

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

    let file = File::open(&path).context("Failed to open data file")?;
    let reader = BufReader::new(file);

    let courses: Vec<Course> = serde_json::from_reader(reader).context("Failed to parse data")?;

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

    let tmp_path = path.with_extension("tmp");
    let file = File::create(&tmp_path).context("Failed to create temp data file")?;
    let mut writer = BufWriter::new(file);

    serde_json::to_writer_pretty(&mut writer, courses).context("Failed to serialize data")?;
    writer.flush().context("Failed to flush data to disk")?;
    drop(writer);

    if let Err(err) = fs::rename(&tmp_path, &path) {
        if path.exists() {
            fs::remove_file(&path).context("Failed to remove existing data file")?;
            fs::rename(&tmp_path, &path).context("Failed to write data file")?;
        } else {
            return Err(err).context("Failed to write data file");
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
}
