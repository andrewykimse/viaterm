use crossterm::event::{KeyCode, KeyModifiers};

/// QMK VIA macro action bytes
const SS_TAP: u8 = 0x01;
const SS_DOWN: u8 = 0x02;
const SS_UP: u8 = 0x03;

/// Map a QMK basic keycode byte to a human-readable name.
fn keycode_name(code: u8) -> &'static str {
    match code {
        0x04 => "KC_A",
        0x05 => "KC_B",
        0x06 => "KC_C",
        0x07 => "KC_D",
        0x08 => "KC_E",
        0x09 => "KC_F",
        0x0A => "KC_G",
        0x0B => "KC_H",
        0x0C => "KC_I",
        0x0D => "KC_J",
        0x0E => "KC_K",
        0x0F => "KC_L",
        0x10 => "KC_M",
        0x11 => "KC_N",
        0x12 => "KC_O",
        0x13 => "KC_P",
        0x14 => "KC_Q",
        0x15 => "KC_R",
        0x16 => "KC_S",
        0x17 => "KC_T",
        0x18 => "KC_U",
        0x19 => "KC_V",
        0x1A => "KC_W",
        0x1B => "KC_X",
        0x1C => "KC_Y",
        0x1D => "KC_Z",
        0x1E => "KC_1",
        0x1F => "KC_2",
        0x20 => "KC_3",
        0x21 => "KC_4",
        0x22 => "KC_5",
        0x23 => "KC_6",
        0x24 => "KC_7",
        0x25 => "KC_8",
        0x26 => "KC_9",
        0x27 => "KC_0",
        0x28 => "KC_ENT",
        0x29 => "KC_ESC",
        0x2A => "KC_BSPC",
        0x2B => "KC_TAB",
        0x2C => "KC_SPC",
        0x2D => "KC_MINS",
        0x2E => "KC_EQL",
        0x2F => "KC_LBRC",
        0x30 => "KC_RBRC",
        0x31 => "KC_BSLS",
        0x33 => "KC_SCLN",
        0x34 => "KC_QUOT",
        0x35 => "KC_GRV",
        0x36 => "KC_COMM",
        0x37 => "KC_DOT",
        0x38 => "KC_SLSH",
        0x39 => "KC_CAPS",
        0x3A => "KC_F1",
        0x3B => "KC_F2",
        0x3C => "KC_F3",
        0x3D => "KC_F4",
        0x3E => "KC_F5",
        0x3F => "KC_F6",
        0x40 => "KC_F7",
        0x41 => "KC_F8",
        0x42 => "KC_F9",
        0x43 => "KC_F10",
        0x44 => "KC_F11",
        0x45 => "KC_F12",
        0xE0 => "KC_LCTL",
        0xE1 => "KC_LSFT",
        0xE2 => "KC_LALT",
        0xE3 => "KC_LGUI",
        0xE4 => "KC_RCTL",
        0xE5 => "KC_RSFT",
        0xE6 => "KC_RALT",
        0xE7 => "KC_RGUI",
        _ => "",
    }
}

/// Reverse lookup: name to keycode byte.
fn keycode_from_name(name: &str) -> Option<u8> {
    // Build from the same mapping
    let pairs: &[(&str, u8)] = &[
        ("KC_A", 0x04), ("KC_B", 0x05), ("KC_C", 0x06), ("KC_D", 0x07),
        ("KC_E", 0x08), ("KC_F", 0x09), ("KC_G", 0x0A), ("KC_H", 0x0B),
        ("KC_I", 0x0C), ("KC_J", 0x0D), ("KC_K", 0x0E), ("KC_L", 0x0F),
        ("KC_M", 0x10), ("KC_N", 0x11), ("KC_O", 0x12), ("KC_P", 0x13),
        ("KC_Q", 0x14), ("KC_R", 0x15), ("KC_S", 0x16), ("KC_T", 0x17),
        ("KC_U", 0x18), ("KC_V", 0x19), ("KC_W", 0x1A), ("KC_X", 0x1B),
        ("KC_Y", 0x1C), ("KC_Z", 0x1D),
        ("KC_1", 0x1E), ("KC_2", 0x1F), ("KC_3", 0x20), ("KC_4", 0x21),
        ("KC_5", 0x22), ("KC_6", 0x23), ("KC_7", 0x24), ("KC_8", 0x25),
        ("KC_9", 0x26), ("KC_0", 0x27),
        ("KC_ENT", 0x28), ("KC_ESC", 0x29), ("KC_BSPC", 0x2A),
        ("KC_TAB", 0x2B), ("KC_SPC", 0x2C),
        ("KC_MINS", 0x2D), ("KC_EQL", 0x2E), ("KC_LBRC", 0x2F),
        ("KC_RBRC", 0x30), ("KC_BSLS", 0x31), ("KC_SCLN", 0x33),
        ("KC_QUOT", 0x34), ("KC_GRV", 0x35), ("KC_COMM", 0x36),
        ("KC_DOT", 0x37), ("KC_SLSH", 0x38), ("KC_CAPS", 0x39),
        ("KC_F1", 0x3A), ("KC_F2", 0x3B), ("KC_F3", 0x3C), ("KC_F4", 0x3D),
        ("KC_F5", 0x3E), ("KC_F6", 0x3F), ("KC_F7", 0x40), ("KC_F8", 0x41),
        ("KC_F9", 0x42), ("KC_F10", 0x43), ("KC_F11", 0x44), ("KC_F12", 0x45),
        ("KC_LCTL", 0xE0), ("KC_LSFT", 0xE1), ("KC_LALT", 0xE2), ("KC_LGUI", 0xE3),
        ("KC_RCTL", 0xE4), ("KC_RSFT", 0xE5), ("KC_RALT", 0xE6), ("KC_RGUI", 0xE7),
    ];
    pairs.iter().find(|(n, _)| *n == name).map(|(_, c)| *c)
}

/// Parse the raw macro byte buffer into individual macro strings.
/// Each macro is null-terminated in the buffer.
/// Printable ASCII (0x04..=0x7F) is stored directly.
/// Special actions: 0x01+keycode (tap), 0x02+keycode (down), 0x03+keycode (up).
pub fn parse_macros(bytes: &[u8], count: usize) -> Vec<String> {
    let mut macros = Vec::new();
    let mut current = String::new();
    let mut i = 0;

    while i < bytes.len() && macros.len() < count {
        match bytes[i] {
            0x00 => {
                macros.push(current.clone());
                current.clear();
            }
            SS_TAP if i + 1 < bytes.len() => {
                i += 1;
                let name = keycode_name(bytes[i]);
                if name.is_empty() {
                    current.push_str(&format!("{{tap:0x{:02X}}}", bytes[i]));
                } else {
                    current.push_str(&format!("{{tap:{name}}}"));
                }
            }
            SS_DOWN if i + 1 < bytes.len() => {
                i += 1;
                let name = keycode_name(bytes[i]);
                if name.is_empty() {
                    current.push_str(&format!("{{down:0x{:02X}}}", bytes[i]));
                } else {
                    current.push_str(&format!("{{down:{name}}}"));
                }
            }
            SS_UP if i + 1 < bytes.len() => {
                i += 1;
                let name = keycode_name(bytes[i]);
                if name.is_empty() {
                    current.push_str(&format!("{{up:0x{:02X}}}", bytes[i]));
                } else {
                    current.push_str(&format!("{{up:{name}}}"));
                }
            }
            c if c >= 0x04 => {
                current.push(c as char);
            }
            _ => {}
        }
        i += 1;
    }

    // Pad with empty macros if we got fewer than expected
    while macros.len() < count {
        macros.push(String::new());
    }

    macros
}

/// Encode macro strings back into the VIA byte buffer format.
/// Returns the full buffer with null separators.
pub fn encode_macros(macros: &[String]) -> Vec<u8> {
    let mut buf = Vec::new();

    for (i, macro_text) in macros.iter().enumerate() {
        let mut chars = macro_text.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '{' {
                // Parse action like {tap:KC_ENT}
                let mut action = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    action.push(c);
                }
                if let Some((kind, key)) = action.split_once(':') {
                    let action_byte = match kind {
                        "tap" => Some(SS_TAP),
                        "down" => Some(SS_DOWN),
                        "up" => Some(SS_UP),
                        _ => None,
                    };
                    let key_byte = if key.starts_with("0x") {
                        u8::from_str_radix(&key[2..], 16).ok()
                    } else {
                        keycode_from_name(key)
                    };
                    if let (Some(ab), Some(kb)) = (action_byte, key_byte) {
                        buf.push(ab);
                        buf.push(kb);
                    }
                }
            } else if (c as u32) >= 0x04 && (c as u32) <= 0x7F {
                buf.push(c as u8);
            }
        }

        // Null-terminate each macro
        if i < macros.len() - 1 {
            buf.push(0x00);
        }
    }

    // Final null terminator
    buf.push(0x00);
    buf
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacroFocus {
    /// Navigating the macro list (left pane)
    List,
    /// Viewing a macro in normal mode (right pane, vim motions)
    Editor,
    /// Typing into a macro (right pane, insert mode)
    Insert,
}

/// Macro editor state.
pub struct MacroState {
    pub macros: Vec<String>,
    pub macro_count: usize,
    pub selected_macro: usize,
    pub focus: MacroFocus,
    pub recording: bool,
    pub cursor_pos: usize,
    pub dirty: bool,
}

impl MacroState {
    pub fn new(macros: Vec<String>, macro_count: usize) -> Self {
        Self {
            macros,
            macro_count,
            selected_macro: 0,
            focus: MacroFocus::List,
            recording: false,
            cursor_pos: 0,
            dirty: false,
        }
    }

    pub fn select_up(&mut self) {
        if self.selected_macro > 0 {
            self.selected_macro -= 1;
        }
    }

    pub fn select_down(&mut self) {
        if self.selected_macro + 1 < self.macros.len() {
            self.selected_macro += 1;
        }
    }

    pub fn focus_editor(&mut self) {
        self.focus = MacroFocus::Editor;
        self.cursor_pos = self.current_macro().len();
    }

    pub fn focus_list(&mut self) {
        self.focus = MacroFocus::List;
    }

    pub fn enter_insert(&mut self) {
        self.focus = MacroFocus::Insert;
        self.cursor_pos = self.current_macro().len();
    }

    pub fn exit_insert(&mut self) {
        self.focus = MacroFocus::Editor;
    }

    pub fn current_macro(&self) -> &str {
        self.macros
            .get(self.selected_macro)
            .map(|s| s.as_str())
            .unwrap_or("")
    }

    pub fn type_char(&mut self, c: char) {
        if let Some(m) = self.macros.get_mut(self.selected_macro) {
            m.insert(self.cursor_pos, c);
            self.cursor_pos += c.len_utf8();
            self.dirty = true;
        }
    }

    pub fn backspace(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        if let Some(m) = self.macros.get_mut(self.selected_macro) {
            if let Some((pos, ch)) = m.char_indices().rev().find(|(i, _)| *i < self.cursor_pos) {
                self.cursor_pos = pos;
                m.remove(pos);
                let _ = ch; // consumed
                self.dirty = true;
            }
        }
    }

    pub fn cursor_left(&mut self) {
        if let Some(m) = self.macros.get(self.selected_macro) {
            if let Some((pos, _)) = m.char_indices().rev().find(|(i, _)| *i < self.cursor_pos) {
                self.cursor_pos = pos;
            }
        }
    }

    pub fn cursor_right(&mut self) {
        if let Some(m) = self.macros.get(self.selected_macro) {
            if let Some((pos, ch)) = m.char_indices().find(|(i, _)| *i >= self.cursor_pos) {
                if pos == self.cursor_pos {
                    self.cursor_pos = pos + ch.len_utf8();
                }
            }
        }
    }

    pub fn clear_current(&mut self) {
        if let Some(m) = self.macros.get_mut(self.selected_macro) {
            m.clear();
            self.cursor_pos = 0;
            self.dirty = true;
        }
    }

    pub fn start_recording(&mut self) {
        self.clear_current();
        self.recording = true;
        self.focus = MacroFocus::Editor;
    }

    pub fn stop_recording(&mut self) {
        self.recording = false;
        self.cursor_pos = self.current_macro().len();
    }

    /// Record a key press, converting it to macro action format.
    pub fn record_key(&mut self, code: KeyCode, modifiers: KeyModifiers) {
        let Some(m) = self.macros.get_mut(self.selected_macro) else {
            return;
        };

        let has_ctrl = modifiers.contains(KeyModifiers::CONTROL);
        let has_shift = modifiers.contains(KeyModifiers::SHIFT);
        let has_alt = modifiers.contains(KeyModifiers::ALT);
        let has_gui = modifiers.contains(KeyModifiers::SUPER);

        // Check if it's a plain printable char with no modifiers (or just shift for uppercase)
        if let KeyCode::Char(c) = code {
            if !has_ctrl && !has_alt && !has_gui {
                // Plain character — insert as literal ASCII
                m.push(c);
                self.cursor_pos = m.len();
                self.dirty = true;
                return;
            }
        }

        // For modified keys or special keys, wrap with modifier down/up
        if has_ctrl {
            m.push_str("{down:KC_LCTL}");
        }
        if has_shift {
            m.push_str("{down:KC_LSFT}");
        }
        if has_alt {
            m.push_str("{down:KC_LALT}");
        }
        if has_gui {
            m.push_str("{down:KC_LGUI}");
        }

        // The main key tap
        if let Some(name) = crossterm_key_to_qmk(&code) {
            m.push_str(&format!("{{tap:{name}}}"));
        }

        // Release modifiers in reverse order
        if has_gui {
            m.push_str("{up:KC_LGUI}");
        }
        if has_alt {
            m.push_str("{up:KC_LALT}");
        }
        if has_shift {
            m.push_str("{up:KC_LSFT}");
        }
        if has_ctrl {
            m.push_str("{up:KC_LCTL}");
        }

        self.cursor_pos = m.len();
        self.dirty = true;
    }
}

/// Map a crossterm KeyCode to a QMK keycode name for macro recording.
fn crossterm_key_to_qmk(code: &KeyCode) -> Option<&'static str> {
    match code {
        KeyCode::Char('a') | KeyCode::Char('A') => Some("KC_A"),
        KeyCode::Char('b') | KeyCode::Char('B') => Some("KC_B"),
        KeyCode::Char('c') | KeyCode::Char('C') => Some("KC_C"),
        KeyCode::Char('d') | KeyCode::Char('D') => Some("KC_D"),
        KeyCode::Char('e') | KeyCode::Char('E') => Some("KC_E"),
        KeyCode::Char('f') | KeyCode::Char('F') => Some("KC_F"),
        KeyCode::Char('g') | KeyCode::Char('G') => Some("KC_G"),
        KeyCode::Char('h') | KeyCode::Char('H') => Some("KC_H"),
        KeyCode::Char('i') | KeyCode::Char('I') => Some("KC_I"),
        KeyCode::Char('j') | KeyCode::Char('J') => Some("KC_J"),
        KeyCode::Char('k') | KeyCode::Char('K') => Some("KC_K"),
        KeyCode::Char('l') | KeyCode::Char('L') => Some("KC_L"),
        KeyCode::Char('m') | KeyCode::Char('M') => Some("KC_M"),
        KeyCode::Char('n') | KeyCode::Char('N') => Some("KC_N"),
        KeyCode::Char('o') | KeyCode::Char('O') => Some("KC_O"),
        KeyCode::Char('p') | KeyCode::Char('P') => Some("KC_P"),
        KeyCode::Char('q') | KeyCode::Char('Q') => Some("KC_Q"),
        KeyCode::Char('r') | KeyCode::Char('R') => Some("KC_R"),
        KeyCode::Char('s') | KeyCode::Char('S') => Some("KC_S"),
        KeyCode::Char('t') | KeyCode::Char('T') => Some("KC_T"),
        KeyCode::Char('u') | KeyCode::Char('U') => Some("KC_U"),
        KeyCode::Char('v') | KeyCode::Char('V') => Some("KC_V"),
        KeyCode::Char('w') | KeyCode::Char('W') => Some("KC_W"),
        KeyCode::Char('x') | KeyCode::Char('X') => Some("KC_X"),
        KeyCode::Char('y') | KeyCode::Char('Y') => Some("KC_Y"),
        KeyCode::Char('z') | KeyCode::Char('Z') => Some("KC_Z"),
        KeyCode::Char('1') | KeyCode::Char('!') => Some("KC_1"),
        KeyCode::Char('2') | KeyCode::Char('@') => Some("KC_2"),
        KeyCode::Char('3') | KeyCode::Char('#') => Some("KC_3"),
        KeyCode::Char('4') | KeyCode::Char('$') => Some("KC_4"),
        KeyCode::Char('5') | KeyCode::Char('%') => Some("KC_5"),
        KeyCode::Char('6') | KeyCode::Char('^') => Some("KC_6"),
        KeyCode::Char('7') | KeyCode::Char('&') => Some("KC_7"),
        KeyCode::Char('8') | KeyCode::Char('*') => Some("KC_8"),
        KeyCode::Char('9') | KeyCode::Char('(') => Some("KC_9"),
        KeyCode::Char('0') | KeyCode::Char(')') => Some("KC_0"),
        KeyCode::Char(' ') => Some("KC_SPC"),
        KeyCode::Char('-') | KeyCode::Char('_') => Some("KC_MINS"),
        KeyCode::Char('=') | KeyCode::Char('+') => Some("KC_EQL"),
        KeyCode::Char('[') | KeyCode::Char('{') => Some("KC_LBRC"),
        KeyCode::Char(']') | KeyCode::Char('}') => Some("KC_RBRC"),
        KeyCode::Char('\\') | KeyCode::Char('|') => Some("KC_BSLS"),
        KeyCode::Char(';') | KeyCode::Char(':') => Some("KC_SCLN"),
        KeyCode::Char('\'') | KeyCode::Char('"') => Some("KC_QUOT"),
        KeyCode::Char('`') | KeyCode::Char('~') => Some("KC_GRV"),
        KeyCode::Char(',') | KeyCode::Char('<') => Some("KC_COMM"),
        KeyCode::Char('.') | KeyCode::Char('>') => Some("KC_DOT"),
        KeyCode::Char('/') | KeyCode::Char('?') => Some("KC_SLSH"),
        KeyCode::Enter => Some("KC_ENT"),
        KeyCode::Backspace => Some("KC_BSPC"),
        KeyCode::Tab | KeyCode::BackTab => Some("KC_TAB"),
        KeyCode::CapsLock => Some("KC_CAPS"),
        KeyCode::F(1) => Some("KC_F1"),
        KeyCode::F(2) => Some("KC_F2"),
        KeyCode::F(3) => Some("KC_F3"),
        KeyCode::F(4) => Some("KC_F4"),
        KeyCode::F(5) => Some("KC_F5"),
        KeyCode::F(6) => Some("KC_F6"),
        KeyCode::F(7) => Some("KC_F7"),
        KeyCode::F(8) => Some("KC_F8"),
        KeyCode::F(9) => Some("KC_F9"),
        KeyCode::F(10) => Some("KC_F10"),
        KeyCode::F(11) => Some("KC_F11"),
        KeyCode::F(12) => Some("KC_F12"),
        KeyCode::Up => Some("KC_UP"),
        KeyCode::Down => Some("KC_DOWN"),
        KeyCode::Left => Some("KC_LEFT"),
        KeyCode::Right => Some("KC_RGHT"),
        KeyCode::Home => Some("KC_HOME"),
        KeyCode::End => Some("KC_END"),
        KeyCode::PageUp => Some("KC_PGUP"),
        KeyCode::PageDown => Some("KC_PGDN"),
        KeyCode::Insert => Some("KC_INS"),
        KeyCode::Delete => Some("KC_DEL"),
        _ => None,
    }
}
