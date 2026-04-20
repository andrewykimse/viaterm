use super::schema::Layouts;

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
/// Filters out decal (decorative) keys since they aren't functional.
pub fn parse_layout(layouts: &Layouts) -> Vec<PositionedKey> {
    layouts
        .keys
        .iter()
        .filter(|k| !k.d) // skip decals
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
