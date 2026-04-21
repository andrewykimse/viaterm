use std::time::Instant;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Paragraph, Widget};

pub struct StatusMessage {
    pub text: String,
    pub style: Style,
    pub created: Instant,
}

impl StatusMessage {
    pub fn info(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Green),
            created: Instant::now(),
        }
    }

    pub fn error(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            style: Style::default().fg(Color::Red),
            created: Instant::now(),
        }
    }

    pub fn is_expired(&self) -> bool {
        self.created.elapsed().as_secs() > 5
    }
}

pub struct StatusBarWidget<'a> {
    pub message: Option<&'a StatusMessage>,
}

impl Widget for StatusBarWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(msg) = self.message
            && !msg.is_expired() {
                Paragraph::new(msg.text.as_str())
                    .style(msg.style)
                    .render(area, buf);
            }
    }
}
