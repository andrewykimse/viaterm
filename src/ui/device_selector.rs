use qmk_via_api::scan::KeyboardDeviceInfo;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget};

pub struct DeviceSelectorWidget<'a> {
    pub devices: &'a [KeyboardDeviceInfo],
    pub selected: usize,
    pub scanning: bool,
}

impl Widget for DeviceSelectorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled("viaterm", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::raw(" — Select a keyboard"),
        ]))
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::DarkGray)));
        title.render(chunks[0], buf);

        // Device list
        if self.scanning {
            Paragraph::new("Scanning for devices...")
                .style(Style::default().fg(Color::Yellow))
                .render(chunks[1], buf);
        } else if self.devices.is_empty() {
            let msg = Paragraph::new(vec![
                Line::from("No VIA-compatible keyboards found."),
                Line::from(""),
                Line::from("Make sure your keyboard:"),
                Line::from("  • Has VIA firmware enabled"),
                Line::from("  • Is connected via USB"),
                Line::from("  • Has proper USB permissions (udev rules on Linux)"),
            ])
            .style(Style::default().fg(Color::Red));
            msg.render(chunks[1], buf);
        } else {
            let items: Vec<ListItem> = self
                .devices
                .iter()
                .enumerate()
                .map(|(i, dev)| {
                    let style = if i == self.selected {
                        Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let text = format!(
                        "  {} (VID:{:04X} PID:{:04X})",
                        dev.product.as_deref().unwrap_or("Unknown"),
                        dev.vendor_id,
                        dev.product_id,
                    );
                    ListItem::new(text).style(style)
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Keyboards"));
            list.render(chunks[1], buf);
        }

        // Help
        let help = Paragraph::new(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Navigate  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" Connect  "),
            Span::styled("r", Style::default().fg(Color::Cyan)),
            Span::raw(" Rescan  "),
            Span::styled("t", Style::default().fg(Color::Cyan)),
            Span::raw(" Key Tester  "),
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(" Quit"),
        ]))
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::DarkGray)));
        help.render(chunks[2], buf);
    }
}
