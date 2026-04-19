use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Widget};

use crate::keyboard::keycodes::{KeycodeCategory, KeycodeEntry, filtered_keycodes, search_keycodes};

/// State for the keycode picker popup.
pub struct KeyPickerState {
    pub active: bool,
    pub category: KeycodeCategory,
    pub search_query: String,
    pub selected_index: usize,
    cached_results: Vec<&'static KeycodeEntry>,
}

impl KeyPickerState {
    pub fn new() -> Self {
        let mut state = Self {
            active: false,
            category: KeycodeCategory::Basic,
            search_query: String::new(),
            selected_index: 0,
            cached_results: Vec::new(),
        };
        state.refresh_results();
        state
    }

    pub fn open(&mut self) {
        self.active = true;
        self.search_query.clear();
        self.selected_index = 0;
        self.category = KeycodeCategory::Basic;
        self.refresh_results();
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn next_category(&mut self) {
        let cats = KeycodeCategory::ALL;
        let idx = cats.iter().position(|c| *c == self.category).unwrap_or(0);
        self.category = cats[(idx + 1) % cats.len()];
        self.selected_index = 0;
        self.refresh_results();
    }

    pub fn prev_category(&mut self) {
        let cats = KeycodeCategory::ALL;
        let idx = cats.iter().position(|c| *c == self.category).unwrap_or(0);
        self.category = cats[idx.checked_sub(1).unwrap_or(cats.len() - 1)];
        self.selected_index = 0;
        self.refresh_results();
    }

    pub fn type_char(&mut self, c: char) {
        self.search_query.push(c);
        self.selected_index = 0;
        self.refresh_results();
    }

    pub fn backspace(&mut self) {
        self.search_query.pop();
        self.selected_index = 0;
        self.refresh_results();
    }

    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.selected_index + 1 < self.cached_results.len() {
            self.selected_index += 1;
        }
    }

    pub fn selected_keycode(&self) -> Option<u16> {
        self.cached_results.get(self.selected_index).map(|e| e.code)
    }

    fn refresh_results(&mut self) {
        self.cached_results = if self.search_query.is_empty() {
            filtered_keycodes(self.category, None)
        } else {
            search_keycodes(&self.search_query)
        };
    }
}

pub struct KeyPickerWidget<'a> {
    pub state: &'a KeyPickerState,
}

impl Widget for KeyPickerWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Center popup in area
        let popup_w = area.width.min(50);
        let popup_h = area.height.min(25);
        let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

        // Clear the popup area
        Clear.render(popup_area, buf);

        let block = Block::default()
            .title(" Assign Keycode ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = block.inner(popup_area);
        block.render(popup_area, buf);

        let chunks = Layout::vertical([
            Constraint::Length(2), // Category tabs
            Constraint::Length(1), // Search input
            Constraint::Min(3),   // Results list
            Constraint::Length(1), // Help
        ])
        .split(inner);

        // Category tabs
        let cat_names: Vec<&str> = KeycodeCategory::ALL.iter().map(|c| c.label()).collect();
        let cat_idx = KeycodeCategory::ALL
            .iter()
            .position(|c| *c == self.state.category)
            .unwrap_or(0);
        let tabs = Tabs::new(cat_names.iter().map(|s| s.to_string()))
            .select(cat_idx)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        tabs.render(chunks[0], buf);

        // Search input
        let search = Paragraph::new(Line::from(vec![
            Span::styled("/ ", Style::default().fg(Color::Cyan)),
            Span::raw(&self.state.search_query),
            Span::styled("_", Style::default().fg(Color::DarkGray)),
        ]));
        search.render(chunks[1], buf);

        // Results
        let visible_height = chunks[2].height as usize;
        let scroll_offset = if self.state.selected_index >= visible_height {
            self.state.selected_index - visible_height + 1
        } else {
            0
        };

        let items: Vec<ListItem> = self
            .state
            .cached_results
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
                let text = format!("{:<6} {}", entry.label, entry.name);
                ListItem::new(text).style(style)
            })
            .collect();

        let list = List::new(items);
        list.render(chunks[2], buf);

        // Help
        let help = Paragraph::new(Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Cyan)),
            Span::raw(" Select  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" Confirm  "),
            Span::styled("←→", Style::default().fg(Color::Cyan)),
            Span::raw(" Category  "),
            Span::styled("Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" Cancel"),
        ]));
        help.render(chunks[3], buf);
    }
}
