use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Widget};

use crate::keyboard::keycodes::{
    KeycodeCategory, KeycodeEntry, MT_MODIFIERS, encode_mt, filtered_keycodes, mt_base_keycodes,
    search_keycodes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickerMode {
    Normal,
    Insert,
}

/// State for the keycode picker popup.
pub struct KeyPickerState {
    pub active: bool,
    pub mode: PickerMode,
    pub pending_g: bool,
    pub count_prefix: Option<u32>,
    pub category: KeycodeCategory,
    pub search_query: String,
    pub selected_index: usize,
    cached_results: Vec<&'static KeycodeEntry>,
    /// For Mod-Tap: selected modifier bits (step 1 done, picking base key).
    pub mt_modifier: Option<u16>,
}

impl KeyPickerState {
    pub fn new() -> Self {
        let mut state = Self {
            active: false,
            mode: PickerMode::Normal,
            pending_g: false,
            count_prefix: None,
            category: KeycodeCategory::Basic,
            search_query: String::new(),
            selected_index: 0,
            cached_results: Vec::new(),
            mt_modifier: None,
        };
        state.refresh_results();
        state
    }

    pub fn open(&mut self) {
        self.active = true;
        self.mode = PickerMode::Normal;
        self.pending_g = false;
        self.count_prefix = None;
        self.search_query.clear();
        self.selected_index = 0;
        self.category = KeycodeCategory::Basic;
        self.mt_modifier = None;
        self.refresh_results();
    }

    pub fn enter_insert(&mut self) {
        self.mode = PickerMode::Insert;
    }

    pub fn enter_normal(&mut self) {
        self.mode = PickerMode::Normal;
    }

    pub fn close(&mut self) {
        self.active = false;
    }

    pub fn next_category(&mut self) {
        let cats = KeycodeCategory::ALL;
        let idx = cats.iter().position(|c| *c == self.category).unwrap_or(0);
        self.category = cats[(idx + 1) % cats.len()];
        self.selected_index = 0;
        self.mt_modifier = None;
        self.refresh_results();
    }

    pub fn prev_category(&mut self) {
        let cats = KeycodeCategory::ALL;
        let idx = cats.iter().position(|c| *c == self.category).unwrap_or(0);
        self.category = cats[idx.checked_sub(1).unwrap_or(cats.len() - 1)];
        self.selected_index = 0;
        self.mt_modifier = None;
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

    pub fn move_top(&mut self) {
        self.selected_index = 0;
    }

    pub fn move_bottom(&mut self) {
        if !self.cached_results.is_empty() {
            self.selected_index = self.cached_results.len() - 1;
        }
    }

    /// Try to confirm the current selection. Returns Some(keycode) if a final
    /// keycode is ready, or None if we advanced to the next MT step.
    pub fn confirm_selection(&mut self) -> Option<u16> {
        let entry = self.cached_results.get(self.selected_index)?;
        if self.category == KeycodeCategory::ModTap && self.mt_modifier.is_none() {
            // Step 1: user picked a modifier — advance to base key selection.
            self.mt_modifier = Some(entry.code);
            self.selected_index = 0;
            self.refresh_results();
            return None;
        }
        if let Some(mod_bits) = self.mt_modifier {
            // Step 2: user picked a base key — encode and return.
            return Some(encode_mt(mod_bits, entry.code));
        }
        Some(entry.code)
    }

    /// Go back from MT base-key selection to modifier selection.
    pub fn mt_back(&mut self) {
        if self.mt_modifier.is_some() {
            self.mt_modifier = None;
            self.selected_index = 0;
            self.refresh_results();
        }
    }

    fn refresh_results(&mut self) {
        if self.category == KeycodeCategory::ModTap {
            self.cached_results = if self.mt_modifier.is_some() {
                // Step 2: show base keycodes
                mt_base_keycodes()
            } else {
                // Step 1: show modifier options
                MT_MODIFIERS.iter().collect()
            };
            return;
        }
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
        let popup_w = area.width.min(70).max(area.width * 3 / 4);
        let popup_h = area.height.min(30).max(area.height * 3 / 4);
        let popup_x = area.x + (area.width.saturating_sub(popup_w)) / 2;
        let popup_y = area.y + (area.height.saturating_sub(popup_h)) / 2;
        let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

        // Clear the popup area
        Clear.render(popup_area, buf);

        let title = if self.state.category == KeycodeCategory::ModTap {
            if let Some(mod_bits) = self.state.mt_modifier {
                let mod_name = crate::keyboard::keycodes::MT_MODIFIERS
                    .iter()
                    .find(|e| e.code == mod_bits)
                    .map(|e| e.label)
                    .unwrap_or("MOD");
                format!(" MT({mod_name}) — Pick Tap Key ")
            } else {
                " Mod-Tap — Pick Modifier ".to_string()
            }
        } else {
            " Assign Keycode ".to_string()
        };
        let block = Block::default()
            .title(title)
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
        let tabs = Tabs::new(cat_names.iter().map(std::string::ToString::to_string))
            .select(cat_idx)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
        tabs.render(chunks[0], buf);

        // Search input
        let search = if self.state.mode == PickerMode::Insert {
            Paragraph::new(Line::from(vec![
                Span::styled("/ ", Style::default().fg(Color::Cyan)),
                Span::raw(&self.state.search_query),
                Span::styled("_", Style::default().fg(Color::DarkGray)),
            ]))
        } else if self.state.search_query.is_empty() {
            Paragraph::new(Line::from(vec![
                Span::styled("/ ", Style::default().fg(Color::DarkGray)),
                Span::styled("search...", Style::default().fg(Color::DarkGray)),
            ]))
        } else {
            Paragraph::new(Line::from(vec![
                Span::styled("/ ", Style::default().fg(Color::DarkGray)),
                Span::raw(&self.state.search_query),
            ]))
        };
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
        let help = if self.state.mode == PickerMode::Normal {
            Paragraph::new(Line::from(vec![
                Span::styled("jk/↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(" Select  "),
                Span::styled("hl/←→", Style::default().fg(Color::Cyan)),
                Span::raw(" Category  "),
                Span::styled("gg/G", Style::default().fg(Color::Cyan)),
                Span::raw(" Top/Bottom  "),
                Span::styled("/", Style::default().fg(Color::Cyan)),
                Span::raw(" Search  "),
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(" Confirm  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" Cancel"),
            ]))
        } else {
            Paragraph::new(Line::from(vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(" Select  "),
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(" Confirm  "),
                Span::styled("←→", Style::default().fg(Color::Cyan)),
                Span::raw(" Category  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" Normal mode"),
            ]))
        };
        help.render(chunks[3], buf);
    }
}
