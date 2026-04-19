use anyhow::{Context, Result};
use kle_serial::Keyboard;

/// A key with its position, size, and matrix coordinates for rendering.
#[derive(Debug, Clone)]
pub struct PositionedKey {
    /// X position in KLE units (1.0 = one standard key width)
    pub x: f64,
    /// Y position in KLE units
    pub y: f64,
    /// Width in KLE units (1.0 for standard, 1.25 for mods, etc.)
    pub w: f64,
    /// Height in KLE units
    pub h: f64,
    /// Matrix row from the keyboard definition
    pub row: u8,
    /// Matrix column from the keyboard definition
    pub col: u8,
    /// Index in the positioned keys list (for selection tracking)
    pub index: usize,
}

/// Parse a VIA keymap JSON array (KLE format) into positioned keys.
///
/// VIA definitions use KLE format with matrix positions encoded in the
/// top-left legend as "row,col" (e.g., "0,0" or "3,14").
pub fn parse_layout(keymap_json: &[serde_json::Value]) -> Result<Vec<PositionedKey>> {
    // KLE expects the keymap to be a JSON array of rows
    let json_str =
        serde_json::to_string(keymap_json).context("Failed to serialize keymap for KLE parsing")?;

    let keyboard: Keyboard =
        serde_json::from_str(&json_str).context("Failed to parse KLE layout")?;

    let mut keys = Vec::new();

    for (i, kle_key) in keyboard.keys.iter().enumerate() {
        // Extract matrix position from legends
        // VIA encodes this in the top-left legend as "row,col"
        let (row, col) = extract_matrix_position(kle_key)
            .with_context(|| format!("Key {} missing matrix position in legends", i))?;

        keys.push(PositionedKey {
            x: kle_key.x,
            y: kle_key.y,
            w: kle_key.width,
            h: kle_key.height,
            row,
            col,
            index: i,
        });
    }

    Ok(keys)
}

/// Extract row,col from a KLE key's legend fields.
/// VIA stores matrix position as "row,col" in one of the legend positions.
fn extract_matrix_position(key: &kle_serial::Key) -> Option<(u8, u8)> {
    // Check all legend positions for a "row,col" pattern
    for legend in &key.legends {
        if let Some(legend) = legend {
            let text = legend.text.trim();
            if let Some((row_str, col_str)) = text.split_once(',') {
                if let (Ok(row), Ok(col)) = (row_str.trim().parse(), col_str.trim().parse()) {
                    return Some((row, col));
                }
            }
        }
    }
    None
}

/// Calculate the bounding box of the entire layout in KLE units.
pub fn layout_bounds(keys: &[PositionedKey]) -> (f64, f64) {
    let max_x = keys
        .iter()
        .map(|k| k.x + k.w)
        .fold(0.0_f64, f64::max);
    let max_y = keys
        .iter()
        .map(|k| k.y + k.h)
        .fold(0.0_f64, f64::max);
    (max_x, max_y)
}
