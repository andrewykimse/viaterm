use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Widget};

use crate::keyboard::lighting::{LightingState, LightingType};

pub struct LightingEditorWidget<'a> {
    pub state: &'a LightingState,
    pub keyboard_name: &'a str,
}

impl LightingEditorWidget<'_> {
    /// Render a horizontal bar gauge for a parameter value.
    fn render_bar(buf: &mut Buffer, area: Rect, value: u8, color: Color) {
        if area.width == 0 || area.height == 0 {
            return;
        }
        let bar_width = area.width;
        let filled = ((value as u32 * bar_width as u32) / 255) as u16;

        for x in 0..bar_width {
            if let Some(cell) = buf.cell_mut((area.x + x, area.y)) {
                if x < filled {
                    cell.set_char('█');
                    cell.set_fg(color);
                } else {
                    cell.set_char('░');
                    cell.set_fg(Color::DarkGray);
                }
            }
        }
    }

    /// Pick a color for the bar based on parameter name and section type.
    fn bar_color(param_name: &str, lighting_type: LightingType, hue: Option<u8>) -> Color {
        match param_name {
            "Hue" => {
                // Show the actual hue color
                let h = hue.unwrap_or(0);
                hue_to_rgb(h)
            }
            "Saturation" => {
                let h = hue.unwrap_or(0);
                hue_to_rgb(h)
            }
            "Brightness" => match lighting_type {
                LightingType::Backlight | LightingType::LedMatrix => Color::White,
                LightingType::RgbLight | LightingType::RgbMatrix => {
                    let h = hue.unwrap_or(0);
                    hue_to_rgb(h)
                }
            },
            _ => Color::Cyan,
        }
    }
}

impl Widget for LightingEditorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Length(2), // Section tabs
            Constraint::Min(5),   // Parameters
            Constraint::Length(3), // Help
        ])
        .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                self.keyboard_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — Lighting"),
            if self.state.dirty {
                Span::styled(" [unsaved]", Style::default().fg(Color::Yellow))
            } else {
                Span::raw("")
            },
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        title.render(chunks[0], buf);

        if self.state.sections.is_empty() {
            let msg = Paragraph::new(Span::styled(
                "  No lighting features detected on this keyboard",
                Style::default().fg(Color::DarkGray),
            ));
            msg.render(chunks[2], buf);
            return;
        }

        // Section tabs
        let tab_names: Vec<String> = self
            .state
            .sections
            .iter()
            .map(|s| format!(" {} ", s.lighting_type.label()))
            .collect();
        let tabs = Tabs::new(tab_names)
            .select(self.state.active_section)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            );
        tabs.render(chunks[1], buf);

        // Parameters
        if let Some(section) = self.state.current_section() {
            let param_area = Block::default()
                .borders(Borders::NONE)
                .inner(chunks[2]);

            // Get hue value for color context (if this section has one)
            let hue = section
                .params
                .iter()
                .find(|p| p.name == "Hue")
                .map(|p| p.value);

            for (i, param) in section.params.iter().enumerate() {
                if i as u16 >= param_area.height {
                    break;
                }

                let y = param_area.y + (i as u16 * 2);
                if y + 1 >= param_area.y + param_area.height {
                    break;
                }

                let is_selected = i == self.state.selected_param;

                // Label row
                let label_area = Rect {
                    x: param_area.x + 1,
                    y,
                    width: param_area.width.saturating_sub(2),
                    height: 1,
                };

                let value_str = format!("{}", param.value);
                let label = Line::from(vec![
                    if is_selected {
                        Span::styled(
                            "▸ ",
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw("  ")
                    },
                    Span::styled(
                        param.name,
                        if is_selected {
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::Gray)
                        },
                    ),
                    Span::raw(" "),
                    Span::styled(
                        value_str,
                        if is_selected {
                            Style::default().fg(Color::Cyan)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                ]);
                Paragraph::new(label).render(label_area, buf);

                // Bar row
                let bar_area = Rect {
                    x: param_area.x + 3,
                    y: y + 1,
                    width: param_area.width.saturating_sub(6),
                    height: 1,
                };

                let color = Self::bar_color(param.name, section.lighting_type, hue);
                Self::render_bar(buf, bar_area, param.value, color);
            }

            // Color swatch for RGB sections
            if matches!(
                section.lighting_type,
                LightingType::RgbLight | LightingType::RgbMatrix
            ) {
                let swatch_y = param_area.y + (section.params.len() as u16 * 2) + 1;
                if swatch_y < param_area.y + param_area.height {
                    let brightness = section.params.first().map(|p| p.value).unwrap_or(255);
                    let h = hue.unwrap_or(0);
                    let sat = section
                        .params
                        .iter()
                        .find(|p| p.name == "Saturation")
                        .map(|p| p.value)
                        .unwrap_or(255);

                    let rgb = hsv_to_rgb(h, sat, brightness);
                    let swatch_area = Rect {
                        x: param_area.x + 3,
                        y: swatch_y,
                        width: param_area.width.saturating_sub(6).min(20),
                        height: 1,
                    };
                    for x in 0..swatch_area.width {
                        if let Some(cell) = buf.cell_mut((swatch_area.x + x, swatch_area.y)) {
                            cell.set_char('█');
                            cell.set_fg(rgb);
                        }
                    }
                }
            }
        }

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled("jk/↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Select  "),
            Span::styled("hl/←→", Style::default().fg(Color::Cyan)),
            Span::raw(" Adjust ±5  "),
            Span::styled("HL/S-←→", Style::default().fg(Color::Cyan)),
            Span::raw(" ±25  "),
            Span::styled("Tab/S-Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" Section  "),
            Span::styled("w", Style::default().fg(Color::Cyan)),
            Span::raw(" Save to EEPROM  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" Back"),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        help.render(chunks[3], buf);
    }
}

/// Convert QMK hue (0-255) to an approximate RGB terminal color.
fn hue_to_rgb(hue: u8) -> Color {
    hsv_to_rgb(hue, 255, 255)
}

/// Convert QMK HSV (all 0-255 range) to terminal RGB color.
fn hsv_to_rgb(hue: u8, sat: u8, val: u8) -> Color {
    // QMK uses 0-255 for hue (not 0-360)
    let h = hue as f32 / 255.0 * 360.0;
    let s = sat as f32 / 255.0;
    let v = val as f32 / 255.0;

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    Color::Rgb(
        ((r1 + m) * 255.0) as u8,
        ((g1 + m) * 255.0) as u8,
        ((b1 + m) * 255.0) as u8,
    )
}
