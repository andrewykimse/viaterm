use std::collections::HashSet;

use crate::definition::layout_parser::PositionedKey;
use crate::definition::schema::MatrixInfo;

pub struct UndoEntry {
    pub layer: u8,
    pub offset: usize,
    pub old_keycode: u16,
    pub new_keycode: u16,
}

/// Editable keymap state with dirty tracking for efficient writes.
pub struct KeymapState {
    /// layers[layer_idx][matrix_offset] = keycode
    /// matrix_offset = row * cols + col
    pub layers: Vec<Vec<u16>>,
    /// Set of (layer, matrix_offset) pairs that have been modified
    dirty: HashSet<(u8, usize)>,
    /// Currently active layer for viewing/editing
    pub active_layer: u8,
    /// Index into the positioned keys list (None = no selection)
    pub selected_key: Option<usize>,
    /// Matrix dimensions
    pub matrix: MatrixInfo,
    /// Undo/redo stacks
    undo_stack: Vec<UndoEntry>,
    redo_stack: Vec<UndoEntry>,
}

impl KeymapState {
    pub fn new(layers: Vec<Vec<u16>>, matrix: MatrixInfo) -> Self {
        Self {
            layers,
            dirty: HashSet::new(),
            active_layer: 0,
            selected_key: None,
            matrix,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn layer_count(&self) -> u8 {
        self.layers.len() as u8
    }

    /// Get the keycode for a specific key on the active layer.
    pub fn get_keycode(&self, key: &PositionedKey) -> u16 {
        let offset = key.row as usize * self.matrix.cols as usize + key.col as usize;
        self.layers
            .get(self.active_layer as usize)
            .and_then(|layer| layer.get(offset).copied())
            .unwrap_or(0)
    }

    /// Set a keycode for a specific key on the active layer.
    pub fn set_keycode(&mut self, key: &PositionedKey, keycode: u16) {
        let offset = key.row as usize * self.matrix.cols as usize + key.col as usize;
        if let Some(layer) = self.layers.get_mut(self.active_layer as usize)
            && let Some(slot) = layer.get_mut(offset)
                && *slot != keycode {
                    let old_keycode = *slot;
                    *slot = keycode;
                    self.dirty.insert((self.active_layer, offset));
                    self.undo_stack.push(UndoEntry {
                        layer: self.active_layer,
                        offset,
                        old_keycode,
                        new_keycode: keycode,
                    });
                    self.redo_stack.clear();
                }
    }

    /// Undo the last keycode change. Returns the (layer, offset) that changed, if any.
    pub fn undo(&mut self) -> Option<(u8, usize)> {
        let entry = self.undo_stack.pop()?;
        if let Some(layer) = self.layers.get_mut(entry.layer as usize)
            && let Some(slot) = layer.get_mut(entry.offset) {
                *slot = entry.old_keycode;
                self.dirty.insert((entry.layer, entry.offset));
                let loc = (entry.layer, entry.offset);
                self.redo_stack.push(entry);
                return Some(loc);
            }
        None
    }

    /// Redo the last undone change. Returns the (layer, offset) that changed, if any.
    pub fn redo(&mut self) -> Option<(u8, usize)> {
        let entry = self.redo_stack.pop()?;
        if let Some(layer) = self.layers.get_mut(entry.layer as usize)
            && let Some(slot) = layer.get_mut(entry.offset) {
                *slot = entry.new_keycode;
                self.dirty.insert((entry.layer, entry.offset));
                let loc = (entry.layer, entry.offset);
                self.undo_stack.push(entry);
                return Some(loc);
            }
        None
    }

    /// Get all dirty entries as (layer, row, col, keycode) for writing to device.
    #[allow(clippy::cast_possible_truncation)]
    pub fn drain_dirty(&mut self) -> Vec<(u8, u8, u8, u16)> {
        let cols = self.matrix.cols as usize;
        self.dirty
            .drain()
            .map(|(layer, offset)| {
                let row = (offset / cols) as u8;
                let col = (offset % cols) as u8;
                let keycode = self.layers[layer as usize][offset];
                (layer, row, col, keycode)
            })
            .collect()
    }

    pub fn has_unsaved_changes(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// Replace all layers from a backup, marking every changed key as dirty.
    pub fn restore_layers(&mut self, new_layers: Vec<Vec<u16>>) {
        for (layer_idx, new_layer) in new_layers.into_iter().enumerate() {
            if let Some(old_layer) = self.layers.get_mut(layer_idx) {
                for (offset, &new_code) in new_layer.iter().enumerate() {
                    if let Some(old_code) = old_layer.get_mut(offset)
                        && *old_code != new_code {
                            *old_code = new_code;
                            self.dirty.insert((layer_idx as u8, offset));
                        }
                }
            }
        }
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// Navigate selection to the next layer.
    pub fn next_layer(&mut self) {
        if self.layer_count() > 0 {
            self.active_layer = (self.active_layer + 1) % self.layer_count();
        }
    }

    /// Navigate selection to the previous layer.
    pub fn prev_layer(&mut self) {
        if self.layer_count() > 0 {
            self.active_layer = self.active_layer.checked_sub(1).unwrap_or(self.layer_count() - 1);
        }
    }

    /// Select the nearest key in the given direction.
    pub fn navigate(&mut self, direction: Direction, keys: &[PositionedKey]) {
        let current = match self.selected_key {
            Some(idx) => &keys[idx],
            None => {
                self.selected_key = Some(0);
                return;
            }
        };

        let cx = current.x + current.w / 2.0;
        let cy = current.y + current.h / 2.0;

        let mut best: Option<(usize, f64)> = None;

        for key in keys {
            if key.index == current.index {
                continue;
            }

            let kx = key.x + key.w / 2.0;
            let ky = key.y + key.h / 2.0;
            let dx = kx - cx;
            let dy = ky - cy;

            // Check if the key is in the right direction
            let in_direction = match direction {
                Direction::Up => dy < -0.1,
                Direction::Down => dy > 0.1,
                Direction::Left => dx < -0.1,
                Direction::Right => dx > 0.1,
            };

            if !in_direction {
                continue;
            }

            // Distance with bias toward the primary axis
            let dist = match direction {
                Direction::Up | Direction::Down => dy.abs() + dx.abs() * 2.0,
                Direction::Left | Direction::Right => dx.abs() + dy.abs() * 2.0,
            };

            if best.is_none_or(|(_, d)| dist < d) {
                best = Some((key.index, dist));
            }
        }

        if let Some((idx, _)) = best {
            self.selected_key = Some(idx);
        }
    }

    /// Jump to the leftmost key on the same row.
    pub fn jump_row_start(&mut self, keys: &[PositionedKey]) {
        let current = match self.selected_key {
            Some(idx) => &keys[idx],
            None => {
                self.selected_key = Some(0);
                return;
            }
        };

        let cy = current.y + current.h / 2.0;
        let threshold = current.h / 2.0;

        let mut best: Option<(usize, f64)> = None;
        for key in keys {
            let ky = key.y + key.h / 2.0;
            if (ky - cy).abs() < threshold {
                let kx = key.x;
                if best.is_none_or(|(_, x)| kx < x) {
                    best = Some((key.index, kx));
                }
            }
        }

        if let Some((idx, _)) = best {
            self.selected_key = Some(idx);
        }
    }

    /// Jump to the rightmost key on the same row.
    pub fn jump_row_end(&mut self, keys: &[PositionedKey]) {
        let current = match self.selected_key {
            Some(idx) => &keys[idx],
            None => {
                self.selected_key = Some(0);
                return;
            }
        };

        let cy = current.y + current.h / 2.0;
        let threshold = current.h / 2.0;

        let mut best: Option<(usize, f64)> = None;
        for key in keys {
            let ky = key.y + key.h / 2.0;
            if (ky - cy).abs() < threshold {
                let kx = key.x + key.w;
                if best.is_none_or(|(_, x)| kx > x) {
                    best = Some((key.index, kx));
                }
            }
        }

        if let Some((idx, _)) = best {
            self.selected_key = Some(idx);
        }
    }

    /// Jump to the topmost key in the same column.
    pub fn jump_col_start(&mut self, keys: &[PositionedKey]) {
        let current = match self.selected_key {
            Some(idx) => &keys[idx],
            None => {
                self.selected_key = Some(0);
                return;
            }
        };

        let cx = current.x + current.w / 2.0;
        let threshold = current.w / 2.0;

        let mut best: Option<(usize, f64)> = None;
        for key in keys {
            let kx = key.x + key.w / 2.0;
            if (kx - cx).abs() < threshold {
                let ky = key.y;
                if best.is_none_or(|(_, y)| ky < y) {
                    best = Some((key.index, ky));
                }
            }
        }

        if let Some((idx, _)) = best {
            self.selected_key = Some(idx);
        }
    }

    /// Jump to the bottommost key in the same column.
    pub fn jump_col_end(&mut self, keys: &[PositionedKey]) {
        let current = match self.selected_key {
            Some(idx) => &keys[idx],
            None => {
                self.selected_key = Some(0);
                return;
            }
        };

        let cx = current.x + current.w / 2.0;
        let threshold = current.w / 2.0;

        let mut best: Option<(usize, f64)> = None;
        for key in keys {
            let kx = key.x + key.w / 2.0;
            if (kx - cx).abs() < threshold {
                let ky = key.y + key.h;
                if best.is_none_or(|(_, y)| ky > y) {
                    best = Some((key.index, ky));
                }
            }
        }

        if let Some((idx, _)) = best {
            self.selected_key = Some(idx);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::layout_parser::PositionedKey;
    use crate::definition::schema::MatrixInfo;

    fn matrix_3x3() -> MatrixInfo {
        MatrixInfo { rows: 3, cols: 3 }
    }

    /// Build a simple 3x3 grid of keys for testing navigation.
    fn keys_3x3() -> Vec<PositionedKey> {
        let mut keys = Vec::new();
        for row in 0..3u8 {
            for col in 0..3u8 {
                let idx = (row as usize) * 3 + col as usize;
                keys.push(PositionedKey {
                    x: col as f64,
                    y: row as f64,
                    w: 1.0,
                    h: 1.0,
                    row,
                    col,
                    index: idx,
                });
            }
        }
        keys
    }

    fn make_keymap() -> KeymapState {
        // 2 layers, 3x3 matrix, distinct keycodes
        let layer0: Vec<u16> = (0..9).map(|i| 0x04 + i).collect();
        let layer1: Vec<u16> = (0..9).map(|i| 0x20 + i).collect();
        let mut km = KeymapState::new(vec![layer0, layer1], matrix_3x3());
        km.selected_key = Some(0);
        km
    }

    // --- Basic state ---

    #[test]
    fn layer_count() {
        let km = make_keymap();
        assert_eq!(km.layer_count(), 2);
    }

    #[test]
    fn initial_layer_is_zero() {
        let km = make_keymap();
        assert_eq!(km.active_layer, 0);
    }

    // --- get/set keycode ---

    #[test]
    fn get_keycode() {
        let km = make_keymap();
        let keys = keys_3x3();
        assert_eq!(km.get_keycode(&keys[0]), 0x04); // row=0, col=0
        assert_eq!(km.get_keycode(&keys[4]), 0x08); // row=1, col=1
    }

    #[test]
    fn set_keycode_marks_dirty() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        assert!(!km.has_unsaved_changes());
        km.set_keycode(&keys[0], 0xFF);
        assert!(km.has_unsaved_changes());
        assert_eq!(km.get_keycode(&keys[0]), 0xFF);
    }

    #[test]
    fn set_same_keycode_is_noop() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        let original = km.get_keycode(&keys[0]);
        km.set_keycode(&keys[0], original);
        assert!(!km.has_unsaved_changes());
    }

    // --- dirty tracking ---

    #[test]
    fn drain_dirty_returns_changes() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.set_keycode(&keys[0], 0xAA);
        km.set_keycode(&keys[4], 0xBB);
        let dirty = km.drain_dirty();
        assert_eq!(dirty.len(), 2);
        assert!(!km.has_unsaved_changes());
    }

    #[test]
    fn drain_dirty_correct_row_col() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        // keys[5] is row=1, col=2
        km.set_keycode(&keys[5], 0xCC);
        let dirty = km.drain_dirty();
        assert_eq!(dirty.len(), 1);
        let (layer, row, col, kc) = dirty[0];
        assert_eq!(layer, 0);
        assert_eq!(row, 1);
        assert_eq!(col, 2);
        assert_eq!(kc, 0xCC);
    }

    // --- undo / redo ---

    #[test]
    fn undo_restores_previous() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        let original = km.get_keycode(&keys[0]);
        km.set_keycode(&keys[0], 0xAA);
        assert_eq!(km.get_keycode(&keys[0]), 0xAA);

        km.undo();
        assert_eq!(km.get_keycode(&keys[0]), original);
    }

    #[test]
    fn redo_reapplies() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.set_keycode(&keys[0], 0xAA);
        km.undo();
        km.redo();
        assert_eq!(km.get_keycode(&keys[0]), 0xAA);
    }

    #[test]
    fn undo_returns_none_when_empty() {
        let mut km = make_keymap();
        assert!(km.undo().is_none());
    }

    #[test]
    fn redo_returns_none_when_empty() {
        let mut km = make_keymap();
        assert!(km.redo().is_none());
    }

    #[test]
    fn new_edit_clears_redo_stack() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.set_keycode(&keys[0], 0xAA);
        km.undo();
        // Now redo stack has one entry
        km.set_keycode(&keys[0], 0xBB);
        // Redo stack should be cleared
        assert!(km.redo().is_none());
    }

    #[test]
    fn multiple_undos() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        let original = km.get_keycode(&keys[0]);
        km.set_keycode(&keys[0], 0xAA);
        km.set_keycode(&keys[0], 0xBB);
        km.set_keycode(&keys[0], 0xCC);

        km.undo();
        assert_eq!(km.get_keycode(&keys[0]), 0xBB);
        km.undo();
        assert_eq!(km.get_keycode(&keys[0]), 0xAA);
        km.undo();
        assert_eq!(km.get_keycode(&keys[0]), original);
    }

    // --- layer switching ---

    #[test]
    fn next_layer_wraps() {
        let mut km = make_keymap();
        assert_eq!(km.active_layer, 0);
        km.next_layer();
        assert_eq!(km.active_layer, 1);
        km.next_layer();
        assert_eq!(km.active_layer, 0); // wraps
    }

    #[test]
    fn prev_layer_wraps() {
        let mut km = make_keymap();
        km.prev_layer();
        assert_eq!(km.active_layer, 1); // wraps to last
        km.prev_layer();
        assert_eq!(km.active_layer, 0);
    }

    #[test]
    fn layer_switch_reads_correct_layer() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        let l0_val = km.get_keycode(&keys[0]);
        km.next_layer();
        let l1_val = km.get_keycode(&keys[0]);
        assert_ne!(l0_val, l1_val);
        assert_eq!(l1_val, 0x20);
    }

    // --- navigation ---

    #[test]
    fn navigate_right() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(0); // top-left
        km.navigate(Direction::Right, &keys);
        assert_eq!(km.selected_key, Some(1));
    }

    #[test]
    fn navigate_down() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(0);
        km.navigate(Direction::Down, &keys);
        assert_eq!(km.selected_key, Some(3)); // row below
    }

    #[test]
    fn navigate_left_from_leftmost_stays() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(0);
        km.navigate(Direction::Left, &keys);
        assert_eq!(km.selected_key, Some(0)); // no key to left
    }

    #[test]
    fn navigate_up_from_topmost_stays() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(1);
        km.navigate(Direction::Up, &keys);
        assert_eq!(km.selected_key, Some(1)); // no key above
    }

    #[test]
    fn navigate_from_none_selects_first() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = None;
        km.navigate(Direction::Right, &keys);
        assert_eq!(km.selected_key, Some(0));
    }

    // --- jump methods ---

    #[test]
    fn jump_row_start() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(5); // row=1, col=2
        km.jump_row_start(&keys);
        assert_eq!(km.selected_key, Some(3)); // row=1, col=0
    }

    #[test]
    fn jump_row_end() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(3); // row=1, col=0
        km.jump_row_end(&keys);
        assert_eq!(km.selected_key, Some(5)); // row=1, col=2
    }

    #[test]
    fn jump_col_start() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(7); // row=2, col=1
        km.jump_col_start(&keys);
        assert_eq!(km.selected_key, Some(1)); // row=0, col=1
    }

    #[test]
    fn jump_col_end() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.selected_key = Some(1); // row=0, col=1
        km.jump_col_end(&keys);
        assert_eq!(km.selected_key, Some(7)); // row=2, col=1
    }

    // --- restore_layers ---

    #[test]
    fn restore_layers_updates_values() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        let new_layer0: Vec<u16> = (0..9).map(|i| 0x50 + i).collect();
        km.restore_layers(vec![new_layer0]);
        assert_eq!(km.get_keycode(&keys[0]), 0x50);
        assert!(km.has_unsaved_changes());
    }

    #[test]
    fn restore_layers_clears_undo() {
        let mut km = make_keymap();
        let keys = keys_3x3();
        km.set_keycode(&keys[0], 0xAA);
        let new_layers = vec![vec![0u16; 9], vec![0u16; 9]];
        km.restore_layers(new_layers);
        assert!(km.undo().is_none());
    }

    #[test]
    fn restore_unchanged_values_not_dirty() {
        let mut km = make_keymap();
        // Restore with exact same values — nothing should be dirty
        let same_layer0: Vec<u16> = (0..9).map(|i| 0x04 + i).collect();
        let same_layer1: Vec<u16> = (0..9).map(|i| 0x20 + i).collect();
        km.restore_layers(vec![same_layer0, same_layer1]);
        assert!(!km.has_unsaved_changes());
    }
}
