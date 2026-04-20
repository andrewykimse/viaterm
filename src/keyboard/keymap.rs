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
        if let Some(layer) = self.layers.get_mut(self.active_layer as usize) {
            if let Some(slot) = layer.get_mut(offset) {
                if *slot != keycode {
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
        }
    }

    /// Undo the last keycode change. Returns the (layer, offset) that changed, if any.
    pub fn undo(&mut self) -> Option<(u8, usize)> {
        let entry = self.undo_stack.pop()?;
        if let Some(layer) = self.layers.get_mut(entry.layer as usize) {
            if let Some(slot) = layer.get_mut(entry.offset) {
                *slot = entry.old_keycode;
                self.dirty.insert((entry.layer, entry.offset));
                let loc = (entry.layer, entry.offset);
                self.redo_stack.push(entry);
                return Some(loc);
            }
        }
        None
    }

    /// Redo the last undone change. Returns the (layer, offset) that changed, if any.
    pub fn redo(&mut self) -> Option<(u8, usize)> {
        let entry = self.redo_stack.pop()?;
        if let Some(layer) = self.layers.get_mut(entry.layer as usize) {
            if let Some(slot) = layer.get_mut(entry.offset) {
                *slot = entry.new_keycode;
                self.dirty.insert((entry.layer, entry.offset));
                let loc = (entry.layer, entry.offset);
                self.undo_stack.push(entry);
                return Some(loc);
            }
        }
        None
    }

    /// Get all dirty entries as (layer, row, col, keycode) for writing to device.
    pub fn drain_dirty(&mut self) -> Vec<(u8, u8, u8, u16)> {
        let cols = self.matrix.cols as usize;
        let entries: Vec<_> = self
            .dirty
            .drain()
            .filter_map(|(layer, offset)| {
                let row = (offset / cols) as u8;
                let col = (offset % cols) as u8;
                let keycode = self.layers[layer as usize][offset];
                Some((layer, row, col, keycode))
            })
            .collect();
        entries
    }

    pub fn has_unsaved_changes(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// Replace all layers from a backup, marking every changed key as dirty.
    pub fn restore_layers(&mut self, new_layers: Vec<Vec<u16>>) {
        for (layer_idx, new_layer) in new_layers.into_iter().enumerate() {
            if let Some(old_layer) = self.layers.get_mut(layer_idx) {
                for (offset, &new_code) in new_layer.iter().enumerate() {
                    if let Some(old_code) = old_layer.get_mut(offset) {
                        if *old_code != new_code {
                            *old_code = new_code;
                            self.dirty.insert((layer_idx as u8, offset));
                        }
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
