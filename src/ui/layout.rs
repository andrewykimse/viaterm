use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::Widget;

use crate::definition::layout_parser::{PositionedKey, layout_bounds};
use crate::keyboard::keycodes::keycode_label;

/// Cells per KLE unit for rendering.
const CELLS_X: f64 = 7.0;
const CELLS_Y: f64 = 3.0;

/// Widget that renders a 2D keyboard layout with box-drawing characters.
pub struct KeyboardLayoutWidget<'a> {
    pub keys: &'a [PositionedKey],
    pub keycodes: &'a [u16],
    pub selected_key: Option<usize>,
    pub cols: u8,
}

impl Widget for KeyboardLayoutWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (layout_w, layout_h) = layout_bounds(self.keys);
        let total_cells_x = (layout_w * CELLS_X).ceil() as u16 + 1;
        let total_cells_y = (layout_h * CELLS_Y).ceil() as u16 + 1;

        // Center the layout in the available area
        let offset_x = area.x + area.width.saturating_sub(total_cells_x) / 2;
        let offset_y = area.y + area.height.saturating_sub(total_cells_y) / 2;

        for key in self.keys {
            let is_selected = self.selected_key == Some(key.index);

            let x1 = offset_x + (key.x * CELLS_X).round() as u16;
            let y1 = offset_y + (key.y * CELLS_Y).round() as u16;
            let x2 = offset_x + ((key.x + key.w) * CELLS_X).round() as u16;
            let y2 = offset_y + ((key.y + key.h) * CELLS_Y).round() as u16;

            // Clip to area
            if x2 <= area.x || y2 <= area.y || x1 >= area.right() || y1 >= area.bottom() {
                continue;
            }

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let border_style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            // Draw box borders
            draw_box(buf, area, x1, y1, x2, y2, border_style, is_selected);

            // Draw keycode label centered in the box
            let matrix_offset =
                key.row as usize * self.cols as usize + key.col as usize;
            let keycode = self.keycodes.get(matrix_offset).copied().unwrap_or(0);
            let label = keycode_label(keycode);

            let inner_w = (x2 - x1).saturating_sub(2) as usize;
            let label_display = if label.len() > inner_w {
                &label[..inner_w]
            } else {
                &label
            };

            let label_x = x1 + 1 + ((inner_w.saturating_sub(label_display.len())) / 2) as u16;
            let label_y = y1 + (y2 - y1) / 2;

            if label_y < area.bottom() && label_x < area.right() {
                buf.set_string(label_x, label_y, label_display, style);
            }
        }
    }
}

fn draw_box(
    buf: &mut Buffer,
    area: Rect,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
    style: Style,
    selected: bool,
) {
    let (h, v, tl, tr, bl, br) = if selected {
        ('═', '║', '╔', '╗', '╚', '╝')
    } else {
        ('─', '│', '┌', '┐', '└', '┘')
    };

    // Top and bottom edges
    for x in x1..=x2 {
        if x < area.right() {
            if y1 >= area.y && y1 < area.bottom() {
                set_char(buf, x, y1, h, style);
            }
            if y2 >= area.y && y2 < area.bottom() {
                set_char(buf, x, y2, h, style);
            }
        }
    }

    // Left and right edges
    for y in y1..=y2 {
        if y < area.bottom() {
            if x1 >= area.x && x1 < area.right() {
                set_char(buf, x1, y, v, style);
            }
            if x2 >= area.x && x2 < area.right() {
                set_char(buf, x2, y, v, style);
            }
        }
    }

    // Corners
    set_char_clipped(buf, area, x1, y1, tl, style);
    set_char_clipped(buf, area, x2, y1, tr, style);
    set_char_clipped(buf, area, x1, y2, bl, style);
    set_char_clipped(buf, area, x2, y2, br, style);
}

fn set_char(buf: &mut Buffer, x: u16, y: u16, ch: char, style: Style) {
    if let Some(cell) = buf.cell_mut((x, y)) {
        cell.set_char(ch);
        cell.set_style(style);
    }
}

fn set_char_clipped(buf: &mut Buffer, area: Rect, x: u16, y: u16, ch: char, style: Style) {
    if x >= area.x && x < area.right() && y >= area.y && y < area.bottom() {
        set_char(buf, x, y, ch, style);
    }
}
