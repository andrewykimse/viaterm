use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget};

use crate::keyboard::backup::BackupEntry;

pub struct BackupPickerState {
    pub active: bool,
    pub entries: Vec<BackupEntry>,
    pub selected_index: usize,
}

impl BackupPickerState {
    pub fn new() -> Self {
        Self {
            active: false,
            entries: Vec::new(),
            selected_index: 0,
        }
    }

    pub fn open(&mut self, entries: Vec<BackupEntry>) {
        self.entries = entries;
        self.selected_index = 0;
        self.active = true;
    }

    pub fn close(&mut self) {
        self.active = false;
        self.entries.clear();
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.entries.len() {
            self.selected_index += 1;
        }
    }

    pub fn selected_entry(&self) -> Option<&BackupEntry> {
        self.entries.get(self.selected_index)
    }
}

pub struct BackupPickerWidget<'a> {
    pub state: &'a BackupPickerState,
}

impl Widget for BackupPickerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let popup_w = area.width.min(70).max(area.width * 3 / 4);
        let popup_h = area.height.min(20).max(area.height / 2);
        let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

        Clear.render(popup_area, buf);

        let block = Block::default()
            .title(" Restore Backup ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let chunks = Layout::vertical([
            Constraint::Min(3),
            Constraint::Length(1),
        ])
        .split(inner);

        if self.state.entries.is_empty() {
            Paragraph::new("No backups found for this keyboard.")
                .style(Style::default().fg(Color::DarkGray))
                .render(chunks[0], buf);
        } else {
            let visible_height = chunks[0].height as usize;
            let scroll_offset = if self.state.selected_index >= visible_height {
                self.state.selected_index - visible_height + 1
            } else {
                0
            };

            let items: Vec<ListItem> = self
                .state
                .entries
                .iter()
                .skip(scroll_offset)
                .take(visible_height)
                .enumerate()
                .map(|(i, entry)| {
                    let actual_idx = i + scroll_offset;
                    let style = if actual_idx == self.state.selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };
                    let name = entry
                        .product_name
                        .as_deref()
                        .unwrap_or("Unknown");
                    let text = format!("  {}  {}", entry.timestamp, name);
                    ListItem::new(text).style(style)
                })
                .collect();

            List::new(items).render(chunks[0], buf);
        }

        let help = Paragraph::new(Line::from(vec![
            Span::styled("jk/↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Select  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" Restore  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" Cancel"),
        ]));
        help.render(chunks[1], buf);
    }
}
