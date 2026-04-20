use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::definition::schema::MatrixInfo;

#[derive(Serialize, Deserialize)]
pub struct KeymapBackup {
    pub version: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub product_name: Option<String>,
    pub timestamp: String,
    pub matrix: MatrixInfo,
    pub layers: Vec<Vec<u16>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macros: Option<Vec<u8>>,
}

/// Info about a backup file for display in the picker.
pub struct BackupEntry {
    pub path: PathBuf,
    pub filename: String,
    pub timestamp: String,
    pub product_name: Option<String>,
}

fn backup_dir() -> Option<PathBuf> {
    ProjectDirs::from("", "", "viaterm").map(|dirs| dirs.cache_dir().join("backups"))
}

/// Save a backup to disk. Returns the file path.
pub fn save_backup(backup: &KeymapBackup) -> Result<PathBuf> {
    let dir = backup_dir().context("Could not determine backup directory")?;
    fs::create_dir_all(&dir)
        .with_context(|| format!("Failed to create backup dir: {}", dir.display()))?;

    let filename = format!(
        "{:04X}_{:04X}_{}.json",
        backup.vendor_id, backup.product_id, backup.timestamp
    );
    let path = dir.join(&filename);

    let json = serde_json::to_string_pretty(backup).context("Failed to serialize backup")?;
    fs::write(&path, json)
        .with_context(|| format!("Failed to write backup: {}", path.display()))?;

    Ok(path)
}

/// Load a backup from a file path.
pub fn load_backup(path: &PathBuf) -> Result<KeymapBackup> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read backup: {}", path.display()))?;
    let backup: KeymapBackup = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse backup: {}", path.display()))?;
    Ok(backup)
}

/// List available backups for a given VID/PID, sorted newest first.
pub fn list_backups(vendor_id: u16, product_id: u16) -> Result<Vec<BackupEntry>> {
    let dir = match backup_dir() {
        Some(d) if d.exists() => d,
        _ => return Ok(Vec::new()),
    };

    let prefix = format!("{:04X}_{:04X}_", vendor_id, product_id);
    let mut entries = Vec::new();

    for entry in fs::read_dir(&dir).context("Failed to read backup directory")? {
        let entry = entry?;
        let filename = entry.file_name().to_string_lossy().to_string();
        if !filename.starts_with(&prefix) || !filename.ends_with(".json") {
            continue;
        }

        // Try to read metadata from the file
        if let Ok(backup) = load_backup(&entry.path()) {
            entries.push(BackupEntry {
                path: entry.path(),
                filename,
                timestamp: backup.timestamp,
                product_name: backup.product_name,
            });
        }
    }

    // Sort newest first
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(entries)
}

/// Validate that a backup is compatible with the current keyboard.
pub fn validate_backup(backup: &KeymapBackup, matrix: &MatrixInfo, layer_count: u8) -> Result<()> {
    if backup.matrix.rows != matrix.rows || backup.matrix.cols != matrix.cols {
        bail!(
            "Matrix mismatch: backup is {}x{}, keyboard is {}x{}",
            backup.matrix.rows,
            backup.matrix.cols,
            matrix.rows,
            matrix.cols
        );
    }
    if backup.layers.len() > layer_count as usize {
        bail!(
            "Backup has {} layers, keyboard supports {}",
            backup.layers.len(),
            layer_count
        );
    }
    Ok(())
}
