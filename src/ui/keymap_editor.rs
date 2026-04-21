use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Widget};

use crate::definition::layout_parser::PositionedKey;
use crate::keyboard::keymap::KeymapState;
use crate::ui::layout::KeyboardLayoutWidget;

pub struct KeymapEditorWidget<'a> {
    pub keymap: &'a KeymapState,
    pub keys: &'a [PositionedKey],
    pub keyboard_name: &'a str,
}

impl Widget for KeymapEditorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Length(3), // Title + connection info
            Constraint::Length(2), // Layer tabs
            Constraint::Min(10),  // Keyboard layout
            Constraint::Length(3), // Help bar
        ])
        .split(area);

        // Title bar
        let unsaved = if self.keymap.has_unsaved_changes() {
            Span::styled(" [modified]", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        };

        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                self.keyboard_name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            unsaved,
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        title.render(chunks[0], buf);

        // Layer tabs
        let layer_names: Vec<String> = (0..self.keymap.layer_count())
            .map(|i| format!(" Layer {} ", i))
            .collect();
        let tabs = Tabs::new(layer_names)
            .select(self.keymap.active_layer as usize)
            .style(Style::default().fg(Color::DarkGray))
            .highlight_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
                    .add_modifier(Modifier::UNDERLINED),
            );
        tabs.render(chunks[1], buf);

        // Keyboard layout
        let active_layer = self.keymap.active_layer as usize;
        let keycodes = self
            .keymap
            .layers
            .get(active_layer)
            .map(|l| l.as_slice())
            .unwrap_or(&[]);

        let layout_widget = KeyboardLayoutWidget {
            keys: self.keys,
            keycodes,
            selected_key: self.keymap.selected_key,
            cols: self.keymap.matrix.cols,
        };
        layout_widget.render(chunks[2], buf);

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled("hjkl/←↑↓→", Style::default().fg(Color::Cyan)),
            Span::raw(" Move  "),
            Span::styled("0/$", Style::default().fg(Color::Cyan)),
            Span::raw(" Row Start/End  "),
            Span::styled("gg/G", Style::default().fg(Color::Cyan)),
            Span::raw(" Col Top/Bottom  "),
            Span::styled("Enter", Style::default().fg(Color::Cyan)),
            Span::raw(" Assign  "),
            Span::styled("u/C-r", Style::default().fg(Color::Cyan)),
            Span::raw(" Undo/Redo  "),
            Span::styled("Tab/S-Tab", Style::default().fg(Color::Cyan)),
            Span::raw(" Layer  "),
            Span::styled("m", Style::default().fg(Color::Cyan)),
            Span::raw(" Macros  "),
            Span::styled("L", Style::default().fg(Color::Cyan)),
            Span::raw(" Lighting  "),
            Span::styled("b/r", Style::default().fg(Color::Cyan)),
            Span::raw(" Backup/Restore  "),
            Span::styled("w", Style::default().fg(Color::Cyan)),
            Span::raw(" Save  "),
            Span::styled("d", Style::default().fg(Color::Cyan)),
            Span::raw(" Disconnect  "),
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(" Quit"),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        help.render(chunks[3], buf);
    }
}
