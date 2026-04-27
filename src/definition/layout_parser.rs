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
/// Applies rotation transforms so that angled layouts (e.g. Alice) render correctly.
pub fn parse_layout(layouts: &Layouts) -> Vec<PositionedKey> {
    let mut all_keys: Vec<&KeyDefinition> = layouts.keys.iter().filter(|k| !k.d).collect();

    // Merge default (option "0") keys from each option group
    for options in layouts.option_keys.values() {
        if let Some(default_keys) = options.get("0") {
            for key in default_keys {
                if !key.d {
                    all_keys.push(key);
                }
            }
        }
    }

    // Compute rotated positions for each key
    let mut positioned: Vec<PositionedKey> = all_keys
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let (x, y) = if k.r != 0.0 {
                // Rotate the key center around (rx, ry), then convert back to top-left
                let cx = k.x + k.w / 2.0;
                let cy = k.y + k.h / 2.0;
                let angle = k.r.to_radians();
                let cos_a = angle.cos();
                let sin_a = angle.sin();
                let dx = cx - k.rx;
                let dy = cy - k.ry;
                let rcx = k.rx + dx * cos_a - dy * sin_a;
                let rcy = k.ry + dx * sin_a + dy * cos_a;
                (rcx - k.w / 2.0, rcy - k.h / 2.0)
            } else {
                (k.x, k.y)
            };
            PositionedKey {
                x,
                y,
                w: k.w,
                h: k.h,
                row: k.row,
                col: k.col,
                index: i,
            }
        })
        .collect();

    // Normalize: shift so minimum x and y are 0
    let min_x = positioned.iter().map(|k| k.x).fold(f64::MAX, f64::min);
    let min_y = positioned.iter().map(|k| k.y).fold(f64::MAX, f64::min);
    if min_x != 0.0 || min_y != 0.0 {
        for k in &mut positioned {
            k.x -= min_x;
            k.y -= min_y;
        }
    }

    // Sort by visual position for consistent navigation order (top-to-bottom, left-to-right)
    positioned.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap()
            .then(a.x.partial_cmp(&b.x).unwrap())
    });

    // Re-index after sorting
    for (i, k) in positioned.iter_mut().enumerate() {
        k.index = i;
    }

    positioned
}

/// Calculate the bounding box of the entire layout in key units.
pub fn layout_bounds(keys: &[PositionedKey]) -> (f64, f64) {
    let max_x = keys.iter().map(|k| k.x + k.w).fold(0.0_f64, f64::max);
    let max_y = keys.iter().map(|k| k.y + k.h).fold(0.0_f64, f64::max);
    (max_x, max_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::schema::{KeyDefinition, Layouts};
    use std::collections::HashMap;

    fn key(row: u8, col: u8, x: f64, y: f64) -> KeyDefinition {
        KeyDefinition {
            row,
            col,
            x,
            y,
            w: 1.0,
            h: 1.0,
            r: 0.0,
            rx: 0.0,
            ry: 0.0,
            d: false,
            color: None,
            ei: None,
        }
    }

    fn make_layouts(keys: Vec<KeyDefinition>) -> Layouts {
        Layouts {
            width: None,
            height: None,
            keys,
            keymap: vec![],
            labels: None,
            option_keys: HashMap::new(),
        }
    }

    // --- parse_layout ---

    #[test]
    fn parse_empty_layout() {
        let layouts = make_layouts(vec![]);
        let keys = parse_layout(&layouts);
        assert!(keys.is_empty());
    }

    #[test]
    fn parse_basic_layout() {
        let layouts = make_layouts(vec![
            key(0, 0, 0.0, 0.0),
            key(0, 1, 1.0, 0.0),
            key(1, 0, 0.0, 1.0),
        ]);
        let keys = parse_layout(&layouts);
        assert_eq!(keys.len(), 3);
        // Should be sorted top-to-bottom, left-to-right
        assert_eq!(keys[0].row, 0);
        assert_eq!(keys[0].col, 0);
        assert_eq!(keys[1].row, 0);
        assert_eq!(keys[1].col, 1);
        assert_eq!(keys[2].row, 1);
        assert_eq!(keys[2].col, 0);
    }

    #[test]
    fn parse_excludes_decals() {
        let mut decal = key(0, 2, 2.0, 0.0);
        decal.d = true;
        let layouts = make_layouts(vec![
            key(0, 0, 0.0, 0.0),
            decal,
        ]);
        let keys = parse_layout(&layouts);
        assert_eq!(keys.len(), 1);
    }

    #[test]
    fn parse_preserves_dimensions() {
        let mut wide = key(0, 0, 0.0, 0.0);
        wide.w = 2.25;
        wide.h = 1.5;
        let layouts = make_layouts(vec![wide]);
        let keys = parse_layout(&layouts);
        assert_eq!(keys[0].w, 2.25);
        assert_eq!(keys[0].h, 1.5);
    }

    #[test]
    fn parse_indices_sequential() {
        let layouts = make_layouts(vec![
            key(0, 0, 0.0, 0.0),
            key(0, 1, 1.0, 0.0),
            key(0, 2, 2.0, 0.0),
        ]);
        let keys = parse_layout(&layouts);
        for (i, k) in keys.iter().enumerate() {
            assert_eq!(k.index, i);
        }
    }

    #[test]
    fn parse_merges_option_keys() {
        let mut layouts = make_layouts(vec![key(0, 0, 0.0, 0.0)]);
        let mut group0 = HashMap::new();
        group0.insert("0".to_string(), vec![key(0, 1, 1.0, 0.0)]);
        group0.insert("1".to_string(), vec![key(0, 1, 1.0, 0.0)]);
        layouts.option_keys.insert("0".to_string(), group0);

        let keys = parse_layout(&layouts);
        // base key + option "0" default key = 2 keys
        assert_eq!(keys.len(), 2);
    }

    #[test]
    fn parse_option_keys_only_default() {
        // Only option "0" should be merged, not option "1"
        let mut layouts = make_layouts(vec![key(0, 0, 0.0, 0.0)]);
        let mut group0 = HashMap::new();
        group0.insert("0".to_string(), vec![key(0, 1, 1.0, 0.0)]);
        group0.insert("1".to_string(), vec![key(0, 2, 2.0, 0.0), key(0, 3, 3.0, 0.0)]);
        layouts.option_keys.insert("0".to_string(), group0);

        let keys = parse_layout(&layouts);
        assert_eq!(keys.len(), 2); // 1 base + 1 from option "0"
    }

    // --- layout_bounds ---

    #[test]
    fn bounds_empty() {
        let (x, y) = layout_bounds(&[]);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn bounds_single_key() {
        let keys = vec![PositionedKey {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
            row: 0,
            col: 0,
            index: 0,
        }];
        let (bx, by) = layout_bounds(&keys);
        assert_eq!(bx, 1.0);
        assert_eq!(by, 1.0);
    }

    #[test]
    fn bounds_multiple_keys() {
        let keys = vec![
            PositionedKey { x: 0.0, y: 0.0, w: 1.0, h: 1.0, row: 0, col: 0, index: 0 },
            PositionedKey { x: 1.0, y: 0.0, w: 2.25, h: 1.0, row: 0, col: 1, index: 1 },
            PositionedKey { x: 0.0, y: 1.0, w: 1.0, h: 2.0, row: 1, col: 0, index: 2 },
        ];
        let (bx, by) = layout_bounds(&keys);
        assert_eq!(bx, 3.25); // 1.0 + 2.25
        assert_eq!(by, 3.0);  // 1.0 + 2.0
    }
}
