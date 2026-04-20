use super::schema::{KeyDefinition, Layouts};

/// A key with its position, size, and matrix coordinates for rendering.
#[derive(Debug, Clone)]
pub struct PositionedKey {
    /// X position in key units (1.0 = one standard key width)
    pub x: f64,
    /// Y position in key units
    pub y: f64,
    /// Width in key units
    pub w: f64,
    /// Height in key units
    pub h: f64,
    /// Matrix row
    pub row: u8,
    /// Matrix column
    pub col: u8,
    /// Index in the positioned keys list (for selection tracking)
    pub index: usize,
}

/// Convert layout key definitions into positioned keys for rendering.
/// Merges in default option keys (option "0" for each group) so that
/// keys like backspace, enter, etc. that live in optionKeys are included.
pub fn parse_layout(layouts: &Layouts) -> Vec<PositionedKey> {
    let mut all_keys: Vec<&KeyDefinition> = layouts.keys.iter().filter(|k| !k.d).collect();

    // Merge default (option "0") keys from each option group
    for (_group, options) in &layouts.option_keys {
        if let Some(default_keys) = options.get("0") {
            for key in default_keys {
                if !key.d {
                    all_keys.push(key);
                }
            }
        }
    }

    // Sort by position for consistent navigation order (top-to-bottom, left-to-right)
    all_keys.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap()
            .then(a.x.partial_cmp(&b.x).unwrap())
    });

    all_keys
        .iter()
        .enumerate()
        .map(|(i, k)| PositionedKey {
            x: k.x,
            y: k.y,
            w: k.w,
            h: k.h,
            row: k.row,
            col: k.col,
            index: i,
        })
        .collect()
}

/// Calculate the bounding box of the entire layout in key units.
pub fn layout_bounds(keys: &[PositionedKey]) -> (f64, f64) {
    let max_x = keys.iter().map(|k| k.x + k.w).fold(0.0_f64, f64::max);
    let max_y = keys.iter().map(|k| k.y + k.h).fold(0.0_f64, f64::max);
    (max_x, max_y)
}
