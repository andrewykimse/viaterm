use std::collections::HashSet;

use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

/// A key in the standard test layout.
#[derive(Debug, Clone)]
pub(crate) struct TestKey {
    label: &'static str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl TestKey {
    const fn new(label: &'static str, x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { label, x, y, w, h }
    }

    const fn unit(label: &'static str, x: f64, y: f64) -> Self {
        Self::new(label, x, y, 1.0, 1.0)
    }
}

/// Standard ANSI keyboard layout (87-key TKL).
fn standard_layout() -> Vec<TestKey> {
    vec![
        // Row 0: Function row
        TestKey::unit("Esc", 0.0, 0.0),
        TestKey::unit("F1", 2.0, 0.0),
        TestKey::unit("F2", 3.0, 0.0),
        TestKey::unit("F3", 4.0, 0.0),
        TestKey::unit("F4", 5.0, 0.0),
        TestKey::unit("F5", 6.5, 0.0),
        TestKey::unit("F6", 7.5, 0.0),
        TestKey::unit("F7", 8.5, 0.0),
        TestKey::unit("F8", 9.5, 0.0),
        TestKey::unit("F9", 11.0, 0.0),
        TestKey::unit("F10", 12.0, 0.0),
        TestKey::unit("F11", 13.0, 0.0),
        TestKey::unit("F12", 14.0, 0.0),
        TestKey::unit("PrtSc", 15.25, 0.0),
        TestKey::unit("ScrLk", 16.25, 0.0),
        TestKey::unit("Pause", 17.25, 0.0),
        // Row 1: Number row
        TestKey::unit("`", 0.0, 1.25),
        TestKey::unit("1", 1.0, 1.25),
        TestKey::unit("2", 2.0, 1.25),
        TestKey::unit("3", 3.0, 1.25),
        TestKey::unit("4", 4.0, 1.25),
        TestKey::unit("5", 5.0, 1.25),
        TestKey::unit("6", 6.0, 1.25),
        TestKey::unit("7", 7.0, 1.25),
        TestKey::unit("8", 8.0, 1.25),
        TestKey::unit("9", 9.0, 1.25),
        TestKey::unit("0", 10.0, 1.25),
        TestKey::unit("-", 11.0, 1.25),
        TestKey::unit("=", 12.0, 1.25),
        TestKey::new("Bksp", 13.0, 1.25, 2.0, 1.0),
        TestKey::unit("Ins", 15.25, 1.25),
        TestKey::unit("Home", 16.25, 1.25),
        TestKey::unit("PgUp", 17.25, 1.25),
        // Row 2: QWERTY row
        TestKey::new("Tab", 0.0, 2.25, 1.5, 1.0),
        TestKey::unit("Q", 1.5, 2.25),
        TestKey::unit("W", 2.5, 2.25),
        TestKey::unit("E", 3.5, 2.25),
        TestKey::unit("R", 4.5, 2.25),
        TestKey::unit("T", 5.5, 2.25),
        TestKey::unit("Y", 6.5, 2.25),
        TestKey::unit("U", 7.5, 2.25),
        TestKey::unit("I", 8.5, 2.25),
        TestKey::unit("O", 9.5, 2.25),
        TestKey::unit("P", 10.5, 2.25),
        TestKey::unit("[", 11.5, 2.25),
        TestKey::unit("]", 12.5, 2.25),
        TestKey::new("\\", 13.5, 2.25, 1.5, 1.0),
        TestKey::unit("Del", 15.25, 2.25),
        TestKey::unit("End", 16.25, 2.25),
        TestKey::unit("PgDn", 17.25, 2.25),
        // Row 3: Home row
        TestKey::new("Caps", 0.0, 3.25, 1.75, 1.0),
        TestKey::unit("A", 1.75, 3.25),
        TestKey::unit("S", 2.75, 3.25),
        TestKey::unit("D", 3.75, 3.25),
        TestKey::unit("F", 4.75, 3.25),
        TestKey::unit("G", 5.75, 3.25),
        TestKey::unit("H", 6.75, 3.25),
        TestKey::unit("J", 7.75, 3.25),
        TestKey::unit("K", 8.75, 3.25),
        TestKey::unit("L", 9.75, 3.25),
        TestKey::unit(";", 10.75, 3.25),
        TestKey::unit("'", 11.75, 3.25),
        TestKey::new("Enter", 12.75, 3.25, 2.25, 1.0),
        // Row 4: Bottom row
        TestKey::new("LShift", 0.0, 4.25, 2.25, 1.0),
        TestKey::unit("Z", 2.25, 4.25),
        TestKey::unit("X", 3.25, 4.25),
        TestKey::unit("C", 4.25, 4.25),
        TestKey::unit("V", 5.25, 4.25),
        TestKey::unit("B", 6.25, 4.25),
        TestKey::unit("N", 7.25, 4.25),
        TestKey::unit("M", 8.25, 4.25),
        TestKey::unit(",", 9.25, 4.25),
        TestKey::unit(".", 10.25, 4.25),
        TestKey::unit("/", 11.25, 4.25),
        TestKey::new("RShift", 12.25, 4.25, 2.75, 1.0),
        TestKey::unit("Up", 16.25, 4.25),
        // Row 5: Space row
        TestKey::new("LCtrl", 0.0, 5.25, 1.25, 1.0),
        TestKey::new("Super", 1.25, 5.25, 1.25, 1.0),
        TestKey::new("LAlt", 2.5, 5.25, 1.25, 1.0),
        TestKey::new("Space", 3.75, 5.25, 6.25, 1.0),
        TestKey::new("RAlt", 10.0, 5.25, 1.25, 1.0),
        TestKey::new("Super", 11.25, 5.25, 1.25, 1.0),
        TestKey::new("Menu", 12.5, 5.25, 1.25, 1.0),
        TestKey::new("RCtrl", 13.75, 5.25, 1.25, 1.0),
        TestKey::unit("Left", 15.25, 5.25),
        TestKey::unit("Down", 16.25, 5.25),
        TestKey::unit("Right", 17.25, 5.25),
    ]
}

/// Map a crossterm KeyCode (+ modifiers) to the label used in the standard layout.
fn keycode_to_label(code: &KeyCode, _modifiers: KeyModifiers) -> Option<&'static str> {
    match code {
        KeyCode::Esc => Some("Esc"),
        KeyCode::F(1) => Some("F1"),
        KeyCode::F(2) => Some("F2"),
        KeyCode::F(3) => Some("F3"),
        KeyCode::F(4) => Some("F4"),
        KeyCode::F(5) => Some("F5"),
        KeyCode::F(6) => Some("F6"),
        KeyCode::F(7) => Some("F7"),
        KeyCode::F(8) => Some("F8"),
        KeyCode::F(9) => Some("F9"),
        KeyCode::F(10) => Some("F10"),
        KeyCode::F(11) => Some("F11"),
        KeyCode::F(12) => Some("F12"),
        KeyCode::Backspace => Some("Bksp"),
        KeyCode::Tab | KeyCode::BackTab => Some("Tab"),
        KeyCode::Enter => Some("Enter"),
        KeyCode::CapsLock => Some("Caps"),
        KeyCode::PrintScreen => Some("PrtSc"),
        KeyCode::ScrollLock => Some("ScrLk"),
        KeyCode::Pause => Some("Pause"),
        KeyCode::Insert => Some("Ins"),
        KeyCode::Home => Some("Home"),
        KeyCode::PageUp => Some("PgUp"),
        KeyCode::Delete => Some("Del"),
        KeyCode::End => Some("End"),
        KeyCode::PageDown => Some("PgDn"),
        KeyCode::Up => Some("Up"),
        KeyCode::Down => Some("Down"),
        KeyCode::Left => Some("Left"),
        KeyCode::Right => Some("Right"),
        KeyCode::Menu => Some("Menu"),
        KeyCode::Char(' ') => Some("Space"),
        KeyCode::Char(c) => match c {
            '`' | '~' => Some("`"),
            '1' | '!' => Some("1"),
            '2' | '@' => Some("2"),
            '3' | '#' => Some("3"),
            '4' | '$' => Some("4"),
            '5' | '%' => Some("5"),
            '6' | '^' => Some("6"),
            '7' | '&' => Some("7"),
            '8' | '*' => Some("8"),
            '9' | '(' => Some("9"),
            '0' | ')' => Some("0"),
            '-' | '_' => Some("-"),
            '=' | '+' => Some("="),
            '[' | '{' => Some("["),
            ']' | '}' => Some("]"),
            '\\' | '|' => Some("\\"),
            ';' | ':' => Some(";"),
            '\'' | '"' => Some("'"),
            ',' | '<' => Some(","),
            '.' | '>' => Some("."),
            '/' | '?' => Some("/"),
            'a'..='z' | 'A'..='Z' => {
                match c.to_ascii_uppercase() {
                    'A' => Some("A"),
                    'B' => Some("B"),
                    'C' => Some("C"),
                    'D' => Some("D"),
                    'E' => Some("E"),
                    'F' => Some("F"),
                    'G' => Some("G"),
                    'H' => Some("H"),
                    'I' => Some("I"),
                    'J' => Some("J"),
                    'K' => Some("K"),
                    'L' => Some("L"),
                    'M' => Some("M"),
                    'N' => Some("N"),
                    'O' => Some("O"),
                    'P' => Some("P"),
                    'Q' => Some("Q"),
                    'R' => Some("R"),
                    'S' => Some("S"),
                    'T' => Some("T"),
                    'U' => Some("U"),
                    'V' => Some("V"),
                    'W' => Some("W"),
                    'X' => Some("X"),
                    'Y' => Some("Y"),
                    'Z' => Some("Z"),
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    }
}

/// Also detect modifier-only presses from the modifiers field.
fn modifier_labels(modifiers: KeyModifiers) -> Vec<&'static str> {
    let mut labels = Vec::new();
    if modifiers.contains(KeyModifiers::SHIFT) {
        // We can't distinguish left/right shift from crossterm, light both
        labels.push("LShift");
        labels.push("RShift");
    }
    if modifiers.contains(KeyModifiers::CONTROL) {
        labels.push("LCtrl");
        labels.push("RCtrl");
    }
    if modifiers.contains(KeyModifiers::ALT) {
        labels.push("LAlt");
        labels.push("RAlt");
    }
    if modifiers.contains(KeyModifiers::SUPER) {
        labels.push("Super");
    }
    labels
}

/// State for the key tester screen.
pub struct KeyTesterState {
    /// Labels of keys that have been pressed.
    pub pressed: HashSet<&'static str>,
    /// The standard layout keys.
    pub layout: Vec<TestKey>,
    /// Whether the last key pressed was Esc (for double-Esc exit).
    pub last_was_esc: bool,
}

impl KeyTesterState {
    pub fn new() -> Self {
        Self {
            pressed: HashSet::new(),
            layout: standard_layout(),
            last_was_esc: false,
        }
    }

    /// Record a key press. Returns true if double-Esc was detected (to exit).
    pub fn register_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> bool {
        // Check for double-Esc
        let is_esc = code == KeyCode::Esc;
        if is_esc && self.last_was_esc {
            return true;
        }
        self.last_was_esc = is_esc;

        // Register modifier keys
        for label in modifier_labels(modifiers) {
            self.pressed.insert(label);
        }

        // Register the main key
        if let Some(label) = keycode_to_label(&code, modifiers) {
            self.pressed.insert(label);
        }

        false
    }

    pub fn reset(&mut self) {
        self.pressed.clear();
        self.last_was_esc = false;
    }
}

/// Widget for rendering the key tester.
pub struct KeyTesterWidget<'a> {
    pub state: &'a KeyTesterState,
}

/// Minimum and preferred cells per key unit for the tester layout.
const MIN_CELLS_X: f64 = 4.0;
const MIN_CELLS_Y: f64 = 2.0;
const PREF_CELLS_X: f64 = 7.0;
const PREF_CELLS_Y: f64 = 3.0;

impl Widget for KeyTesterWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(5),
            Constraint::Length(3),
        ])
        .split(area);

        // Title
        let pressed_count = self.state.pressed.len();
        let total_count = self.state.layout.len();
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "viaterm",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" — Key Tester"),
            Span::styled(
                format!("  ({pressed_count}/{total_count} keys tested)"),
                Style::default().fg(Color::DarkGray),
            ),
        ]))
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        title.render(chunks[0], buf);

        // Keyboard layout area
        render_test_layout(self.state, chunks[1], buf);

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled("Press any key", Style::default().fg(Color::Cyan)),
            Span::raw(" to test  "),
            Span::styled("Ctrl+R", Style::default().fg(Color::Cyan)),
            Span::raw(" Reset  "),
            Span::styled("Esc Esc", Style::default().fg(Color::Cyan)),
            Span::raw(" Back"),
        ]))
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(Color::DarkGray)),
        );
        help.render(chunks[2], buf);
    }
}

fn render_test_layout(state: &KeyTesterState, area: Rect, buf: &mut Buffer) {
    let keys = &state.layout;
    if keys.is_empty() || area.width < 4 || area.height < 2 {
        return;
    }

    // Compute bounds
    let max_x = keys.iter().map(|k| k.x + k.w).fold(0.0_f64, f64::max);
    let max_y = keys.iter().map(|k| k.y + k.h).fold(0.0_f64, f64::max);
    if max_x == 0.0 || max_y == 0.0 {
        return;
    }

    let available_w = (area.width.saturating_sub(1)) as f64;
    let available_h = (area.height.saturating_sub(1)) as f64;

    let scale_x = (available_w / max_x).clamp(MIN_CELLS_X, PREF_CELLS_X);
    let scale_y = (available_h / max_y).clamp(MIN_CELLS_Y, PREF_CELLS_Y);

    let total_cells_x = (max_x * scale_x).ceil() as u16 + 1;
    let total_cells_y = (max_y * scale_y).ceil() as u16 + 1;

    let offset_x = area.x + area.width.saturating_sub(total_cells_x) / 2;
    let offset_y = area.y + area.height.saturating_sub(total_cells_y) / 2;

    for key in keys {
        let is_pressed = state.pressed.contains(key.label);

        let x1 = offset_x + (key.x * scale_x).round() as u16;
        let y1 = offset_y + (key.y * scale_y).round() as u16;
        let x2 = offset_x + ((key.x + key.w) * scale_x).round() as u16;
        let y2 = offset_y + ((key.y + key.h) * scale_y).round() as u16;

        if x2 <= area.x || y2 <= area.y || x1 >= area.right() || y1 >= area.bottom() {
            continue;
        }

        let style = if is_pressed {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let border_style = if is_pressed {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        draw_box(buf, area, x1, y1, x2, y2, border_style, is_pressed);

        // Label
        let inner_w = (x2.saturating_sub(x1)).saturating_sub(2) as usize;
        if inner_w == 0 {
            continue;
        }
        let label = if key.label.len() > inner_w {
            &key.label[..inner_w]
        } else {
            key.label
        };

        let label_x = x1 + 1 + ((inner_w.saturating_sub(label.len())) / 2) as u16;
        let label_y = y1 + (y2.saturating_sub(y1)) / 2;

        if label_y < area.bottom() && label_x < area.right() {
            buf.set_string(label_x, label_y, label, style);
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
    thick: bool,
) {
    let (h, v, tl, tr, bl, br) = if thick {
        ('═', '║', '╔', '╗', '╚', '╝')
    } else {
        ('─', '│', '┌', '┐', '└', '┘')
    };

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
