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

    let prefix = format!("{vendor_id:04X}_{product_id:04X}_");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_backup(rows: u8, cols: u8, num_layers: usize) -> KeymapBackup {
        KeymapBackup {
            version: 1,
            vendor_id: 0x1234,
            product_id: 0x5678,
            product_name: Some("Test KB".to_string()),
            timestamp: "2026-04-21T120000".to_string(),
            matrix: MatrixInfo { rows, cols },
            layers: (0..num_layers)
                .map(|_| vec![0u16; (rows as usize) * (cols as usize)])
                .collect(),
            macros: None,
        }
    }

    // --- validate_backup ---

    #[test]
    fn validate_matching_backup() {
        let backup = make_backup(4, 12, 2);
        let matrix = MatrixInfo { rows: 4, cols: 12 };
        assert!(validate_backup(&backup, &matrix, 4).is_ok());
    }

    #[test]
    fn validate_fewer_layers_ok() {
        let backup = make_backup(4, 12, 2);
        let matrix = MatrixInfo { rows: 4, cols: 12 };
        // Backup has 2 layers, keyboard supports 4 — fine
        assert!(validate_backup(&backup, &matrix, 4).is_ok());
    }

    #[test]
    fn validate_exact_layers_ok() {
        let backup = make_backup(4, 12, 4);
        let matrix = MatrixInfo { rows: 4, cols: 12 };
        assert!(validate_backup(&backup, &matrix, 4).is_ok());
    }

    #[test]
    fn validate_too_many_layers_fails() {
        let backup = make_backup(4, 12, 5);
        let matrix = MatrixInfo { rows: 4, cols: 12 };
        let err = validate_backup(&backup, &matrix, 4);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("layers"));
    }

    #[test]
    fn validate_row_mismatch_fails() {
        let backup = make_backup(5, 12, 2);
        let matrix = MatrixInfo { rows: 4, cols: 12 };
        let err = validate_backup(&backup, &matrix, 4);
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("Matrix mismatch"));
    }

    #[test]
    fn validate_col_mismatch_fails() {
        let backup = make_backup(4, 14, 2);
        let matrix = MatrixInfo { rows: 4, cols: 12 };
        assert!(validate_backup(&backup, &matrix, 4).is_err());
    }

    // --- serialization roundtrip ---

    #[test]
    fn backup_serde_roundtrip() {
        let backup = make_backup(4, 12, 2);
        let json = serde_json::to_string(&backup).unwrap();
        let restored: KeymapBackup = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.version, backup.version);
        assert_eq!(restored.vendor_id, backup.vendor_id);
        assert_eq!(restored.product_id, backup.product_id);
        assert_eq!(restored.matrix.rows, 4);
        assert_eq!(restored.matrix.cols, 12);
        assert_eq!(restored.layers.len(), 2);
        assert_eq!(restored.timestamp, backup.timestamp);
    }

    #[test]
    fn backup_with_macros_roundtrip() {
        let mut backup = make_backup(2, 3, 1);
        backup.macros = Some(vec![0x68, 0x69, 0x00]); // "hi\0"
        let json = serde_json::to_string(&backup).unwrap();
        let restored: KeymapBackup = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.macros, Some(vec![0x68, 0x69, 0x00]));
    }

    #[test]
    fn backup_without_macros_omits_field() {
        let backup = make_backup(2, 3, 1);
        let json = serde_json::to_string(&backup).unwrap();
        assert!(!json.contains("macros"));
    }
}
