use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap};

use crate::keyboard::macros::MacroState;

pub struct MacroEditorWidget<'a> {
    pub state: &'a MacroState,
    pub keyboard_name: &'a str,
}

impl Widget for MacroEditorWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Length(3), // Title
            Constraint::Min(5),   // Content
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
            Span::raw(" — Macro Editor"),
            if self.state.recording {
                Span::styled(" [RECORDING]", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
            } else if self.state.dirty {
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

        // Content: macro list on left, editor on right
        let content_chunks = Layout::horizontal([
            Constraint::Length(16), // Macro list
            Constraint::Min(20),   // Editor
        ])
        .split(chunks[1]);

        // Macro list
        let items: Vec<ListItem> = (0..self.state.macros.len())
            .map(|i| {
                let style = if i == self.state.selected_macro {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if !self.state.macros[i].is_empty() {
                    Style::default().fg(Color::White)
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                let label = format!("  M{i:<3} {}", if self.state.macros[i].is_empty() { "" } else { "*" });
                ListItem::new(label).style(style)
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .borders(Borders::RIGHT)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        list.render(content_chunks[0], buf);

        // Editor pane
        let editor_block = Block::default()
            .title(format!(" M{} ", self.state.selected_macro))
            .borders(Borders::ALL)
            .border_style(if self.state.recording {
                Style::default().fg(Color::Red)
            } else if self.state.editing {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            });
        let editor_inner = editor_block.inner(content_chunks[1]);
        editor_block.render(content_chunks[1], buf);

        let macro_text = self.state.current_macro();
        if self.state.recording {
            // Show recorded content with blinking cursor
            let content = Paragraph::new(Line::from(vec![
                Span::raw(macro_text),
                Span::styled("_", Style::default().fg(Color::Red)),
            ]))
            .wrap(Wrap { trim: false });
            content.render(editor_inner, buf);
        } else if self.state.editing {
            // Show text with cursor
            let (before, after) = macro_text.split_at(
                self.state.cursor_pos.min(macro_text.len()),
            );
            let cursor_char = after.chars().next().unwrap_or(' ');
            let after_cursor = if after.is_empty() {
                ""
            } else {
                &after[cursor_char.len_utf8()..]
            };
            let content = Paragraph::new(Line::from(vec![
                Span::raw(before),
                Span::styled(
                    cursor_char.to_string(),
                    Style::default().fg(Color::Black).bg(Color::White),
                ),
                Span::raw(after_cursor),
            ]))
            .wrap(Wrap { trim: false });
            content.render(editor_inner, buf);
        } else if macro_text.is_empty() {
            Paragraph::new(Span::styled(
                "Empty — press Enter to edit",
                Style::default().fg(Color::DarkGray),
            ))
            .render(editor_inner, buf);
        } else {
            Paragraph::new(macro_text)
                .wrap(Wrap { trim: false })
                .render(editor_inner, buf);
        }

        // Help bar
        let help = if self.state.recording {
            Paragraph::new(Line::from(vec![
                Span::styled("Press keys", Style::default().fg(Color::Red)),
                Span::raw(" to record  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(" Stop recording"),
            ]))
        } else if self.state.editing {
            Paragraph::new(Line::from(vec![
                Span::styled("Type", Style::default().fg(Color::Cyan)),
                Span::raw(" to add text  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" Stop editing  "),
                Span::styled("Backspace", Style::default().fg(Color::Cyan)),
                Span::raw(" Delete"),
            ]))
        } else {
            Paragraph::new(Line::from(vec![
                Span::styled("jk/↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(" Select  "),
                Span::styled("Enter", Style::default().fg(Color::Cyan)),
                Span::raw(" Edit  "),
                Span::styled("R", Style::default().fg(Color::Cyan)),
                Span::raw(" Record  "),
                Span::styled("dd", Style::default().fg(Color::Cyan)),
                Span::raw(" Clear  "),
                Span::styled("w", Style::default().fg(Color::Cyan)),
                Span::raw(" Save  "),
                Span::styled("Esc", Style::default().fg(Color::Cyan)),
                Span::raw(" Back"),
            ]))
        };
        help.block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        )
        .render(chunks[2], buf);
    }
}
