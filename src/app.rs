use std::path::PathBuf;

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use qmk_via_api::scan::KeyboardDeviceInfo;
use ratatui::Frame;

use crate::definition::layout_parser::{PositionedKey, parse_layout};
use crate::definition::loader::{fetch_definition, load_definition_file};
use crate::definition::schema::ViaDefinition;
use crate::event::is_quit;
use crate::keyboard::backup::{self, KeymapBackup};
use crate::keyboard::connection::{KeyboardConnection, scan_devices};
use crate::keyboard::keymap::KeymapState;
use crate::keyboard::keymap::Direction;
use crate::keyboard::lighting::{self, LightingState};
use crate::keyboard::macros::{self, MacroState};
use crate::ui::backup_picker::{BackupPickerState, BackupPickerWidget};
use crate::ui::device_selector::DeviceSelectorWidget;
use crate::ui::key_picker::{KeyPickerState, KeyPickerWidget};
use crate::ui::key_tester::{KeyTesterState, KeyTesterWidget};
use crate::ui::keymap_editor::KeymapEditorWidget;
use crate::ui::lighting_editor::LightingEditorWidget;
use crate::ui::macro_editor::MacroEditorWidget;
use crate::ui::status_bar::{StatusBarWidget, StatusMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    DeviceSelect,
    KeymapEditor,
    KeyTester,
    MacroEditor,
    LightingEditor,
}

pub struct App {
    pub screen: Screen,
    pub should_quit: bool,

    // Device selection
    pub devices: Vec<KeyboardDeviceInfo>,
    pub device_selected: usize,

    // Connection
    pub connection: Option<KeyboardConnection>,
    pub definition: Option<ViaDefinition>,
    pub positioned_keys: Vec<PositionedKey>,
    pub keymap: Option<KeymapState>,

    // Editor vim state
    pub pending_g: bool,
    pub count_prefix: Option<u32>,

    // Key picker
    pub key_picker: KeyPickerState,

    // Backup picker
    pub backup_picker: BackupPickerState,

    // Macro editor
    pub macro_state: Option<MacroState>,
    pub pending_d: bool,

    // Lighting editor
    pub lighting_state: Option<LightingState>,

    // Key tester
    pub key_tester: KeyTesterState,

    // Status
    pub status: Option<StatusMessage>,

    // Config
    pub definition_path: Option<PathBuf>,
}

impl App {
    pub fn new(definition_path: Option<PathBuf>) -> Self {
        Self {
            screen: Screen::DeviceSelect,
            should_quit: false,
            devices: Vec::new(),
            device_selected: 0,
            connection: None,
            definition: None,
            positioned_keys: Vec::new(),
            keymap: None,
            pending_g: false,
            count_prefix: None,
            key_picker: KeyPickerState::new(),
            backup_picker: BackupPickerState::new(),
            macro_state: None,
            pending_d: false,
            lighting_state: None,
            key_tester: KeyTesterState::new(),
            status: None,
            definition_path,
        }
    }

    pub fn scan(&mut self) {
        match scan_devices() {
            Ok(devices) => {
                self.devices = devices;
                self.device_selected = 0;
                if self.devices.is_empty() {
                    self.status = Some(StatusMessage::error("No keyboards found"));
                } else {
                    self.status = Some(StatusMessage::info(format!(
                        "Found {} keyboard(s)",
                        self.devices.len()
                    )));
                }
            }
            Err(e) => {
                self.status = Some(StatusMessage::error(format!("Scan failed: {e}")));
            }
        }
    }

    pub fn connect_selected(&mut self) -> Result<()> {
        let device = self
            .devices
            .get(self.device_selected)
            .context("No device selected")?
            .clone();

        // Connect to device first (we need VID/PID and protocol version)
        let conn = KeyboardConnection::connect(&device)?;

        // Load definition: from file if provided, otherwise auto-fetch by VID/PID
        let definition = if let Some(def_path) = &self.definition_path {
            load_definition_file(def_path)?
        } else {
            self.status = Some(StatusMessage::info(format!(
                "Fetching definition for {:04X}:{:04X}...",
                device.vendor_id, device.product_id
            )));
            fetch_definition(device.vendor_id, device.product_id, conn.protocol_version)?
        };

        self.status = Some(StatusMessage::info(format!(
            "Connected: {} (protocol v{})",
            definition.name, conn.protocol_version
        )));

        // Read keymap
        let layers = conn.read_all_layers(&definition.matrix)?;

        // Parse layout
        let keys = parse_layout(&definition.layouts);

        let mut keymap = KeymapState::new(layers, definition.matrix.clone());
        if !keys.is_empty() {
            keymap.selected_key = Some(0);
        }

        self.connection = Some(conn);
        self.positioned_keys = keys;
        self.keymap = Some(keymap);
        self.definition = Some(definition);
        self.screen = Screen::KeymapEditor;

        Ok(())
    }

    pub fn disconnect(&mut self) {
        self.connection = None;
        self.keymap = None;
        self.definition = None;
        self.macro_state = None;
        self.lighting_state = None;
        self.positioned_keys.clear();
        self.screen = Screen::DeviceSelect;
        self.status = Some(StatusMessage::info("Disconnected"));
    }

    pub fn save_keymap(&mut self) -> Result<()> {
        let conn = self
            .connection
            .as_ref()
            .context("Not connected to a keyboard")?;
        let keymap = self
            .keymap
            .as_mut()
            .context("No keymap loaded")?;

        let dirty = keymap.drain_dirty();
        if dirty.is_empty() {
            self.status = Some(StatusMessage::info("No changes to save"));
            return Ok(());
        }

        let count = dirty.len();
        for (layer, row, col, keycode) in dirty {
            conn.set_keycode(layer, row, col, keycode)?;
        }

        self.status = Some(StatusMessage::info(format!("Saved {count} key(s)")));
        Ok(())
    }

    fn create_backup(&mut self) {
        let conn = match self.connection.as_ref() {
            Some(c) => c,
            None => return,
        };
        let keymap = match self.keymap.as_ref() {
            Some(k) => k,
            None => return,
        };

        let macro_bytes = conn.read_macro_bytes().ok();
        let now = chrono::Local::now().format("%Y-%m-%dT%H%M%S").to_string();

        let backup = KeymapBackup {
            version: 1,
            vendor_id: conn.device_info.vendor_id,
            product_id: conn.device_info.product_id,
            product_name: conn.device_info.product.clone(),
            timestamp: now,
            matrix: keymap.matrix.clone(),
            layers: keymap.layers.clone(),
            macros: macro_bytes,
        };

        match backup::save_backup(&backup) {
            Ok(path) => {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                self.status = Some(StatusMessage::info(format!("Backup saved: {filename}")));
            }
            Err(e) => {
                self.status = Some(StatusMessage::error(format!("Backup failed: {e}")));
            }
        }
    }

    fn open_restore_picker(&mut self) {
        let conn = match self.connection.as_ref() {
            Some(c) => c,
            None => return,
        };

        match backup::list_backups(conn.device_info.vendor_id, conn.device_info.product_id) {
            Ok(entries) => {
                if entries.is_empty() {
                    self.status =
                        Some(StatusMessage::info("No backups found for this keyboard"));
                } else {
                    self.backup_picker.open(entries);
                }
            }
            Err(e) => {
                self.status = Some(StatusMessage::error(format!("Failed to list backups: {e}")));
            }
        }
    }

    fn restore_selected_backup(&mut self) {
        let path = match self.backup_picker.selected_entry() {
            Some(entry) => entry.path.clone(),
            None => return,
        };
        self.backup_picker.close();

        let loaded = match backup::load_backup(&path) {
            Ok(b) => b,
            Err(e) => {
                self.status = Some(StatusMessage::error(format!("Restore failed: {e}")));
                return;
            }
        };

        let keymap = match self.keymap.as_mut() {
            Some(k) => k,
            None => return,
        };

        if let Err(e) = backup::validate_backup(&loaded, &keymap.matrix, keymap.layer_count()) {
            self.status = Some(StatusMessage::error(format!("Restore failed: {e}")));
            return;
        }

        // Restore layers — mark all changed keys dirty
        keymap.restore_layers(loaded.layers);

        // Restore macros directly to device if present
        if let Some(macro_bytes) = loaded.macros {
            if let Some(conn) = self.connection.as_ref() {
                if let Err(e) = conn.write_macro_bytes(macro_bytes) {
                    self.status =
                        Some(StatusMessage::error(format!("Macro restore failed: {e}")));
                    return;
                }
            }
        }

        self.status = Some(StatusMessage::info(
            "Backup restored (press w to save keymaps to device)",
        ));
    }

    fn open_macro_editor(&mut self) {
        let conn = match self.connection.as_ref() {
            Some(c) => c,
            None => return,
        };

        let macro_count = match conn.get_macro_count() {
            Ok(c) => c as usize,
            Err(e) => {
                self.status =
                    Some(StatusMessage::error(format!("Failed to read macro count: {e}")));
                return;
            }
        };

        let macro_bytes = match conn.read_macro_bytes() {
            Ok(b) => b,
            Err(e) => {
                self.status =
                    Some(StatusMessage::error(format!("Failed to read macros: {e}")));
                return;
            }
        };

        let parsed = macros::parse_macros(&macro_bytes, macro_count);
        self.macro_state = Some(MacroState::new(parsed, macro_count));
        self.screen = Screen::MacroEditor;
    }

    fn save_macros(&mut self) {
        let macro_state = match self.macro_state.as_mut() {
            Some(s) => s,
            None => return,
        };

        if !macro_state.dirty {
            self.status = Some(StatusMessage::info("No macro changes to save"));
            return;
        }

        let encoded = macros::encode_macros(&macro_state.macros);

        match self.connection.as_ref() {
            Some(conn) => match conn.write_macro_bytes(encoded) {
                Ok(()) => {
                    macro_state.dirty = false;
                    self.status = Some(StatusMessage::info("Macros saved"));
                }
                Err(e) => {
                    self.status =
                        Some(StatusMessage::error(format!("Macro save failed: {e}")));
                }
            },
            None => {
                self.status = Some(StatusMessage::error("Not connected"));
            }
        }
    }

    fn open_lighting_editor(&mut self) {
        let conn = match self.connection.as_ref() {
            Some(c) => c,
            None => return,
        };

        match lighting::detect_lighting(conn) {
            Ok(state) => {
                if state.sections.is_empty() {
                    self.status =
                        Some(StatusMessage::info("No lighting features detected"));
                } else {
                    let names: Vec<&str> =
                        state.sections.iter().map(|s| s.lighting_type.label()).collect();
                    self.status = Some(StatusMessage::info(format!(
                        "Detected: {}",
                        names.join(", ")
                    )));
                    self.lighting_state = Some(state);
                    self.screen = Screen::LightingEditor;
                }
            }
            Err(e) => {
                self.status =
                    Some(StatusMessage::error(format!("Failed to read lighting: {e}")));
            }
        }
    }

    fn save_lighting_settings(&mut self) {
        let state = match self.lighting_state.as_mut() {
            Some(s) => s,
            None => return,
        };

        if !state.dirty {
            self.status = Some(StatusMessage::info("No lighting changes to save"));
            return;
        }

        match self.connection.as_ref() {
            Some(conn) => match lighting::save_lighting(conn) {
                Ok(()) => {
                    state.dirty = false;
                    self.status = Some(StatusMessage::info("Lighting saved to EEPROM"));
                }
                Err(e) => {
                    self.status =
                        Some(StatusMessage::error(format!("Lighting save failed: {e}")));
                }
            },
            None => {
                self.status = Some(StatusMessage::error("Not connected"));
            }
        }
    }

    /// Push current lighting values to the keyboard for live preview.
    fn apply_lighting_live(&self) {
        if let (Some(conn), Some(state)) =
            (self.connection.as_ref(), self.lighting_state.as_ref())
        {
            let _ = lighting::apply_lighting(conn, state);
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if self.backup_picker.active {
                self.handle_backup_picker_key(key);
            } else if self.key_picker.active {
                self.handle_picker_key(key);
            } else {
                match self.screen {
                    Screen::DeviceSelect => self.handle_device_select_key(key),
                    Screen::KeymapEditor => self.handle_editor_key(key),
                    Screen::KeyTester => self.handle_key_tester_key(key),
                    Screen::MacroEditor => self.handle_macro_editor_key(key),
                    Screen::LightingEditor => self.handle_lighting_editor_key(key),
                }
            }
        }
    }

    fn handle_device_select_key(&mut self, key: KeyEvent) {
        if is_quit(&key) {
            self.should_quit = true;
            return;
        }

        match key.code {
            KeyCode::Up => {
                if self.device_selected > 0 {
                    self.device_selected -= 1;
                }
            }
            KeyCode::Down => {
                if self.device_selected + 1 < self.devices.len() {
                    self.device_selected += 1;
                }
            }
            KeyCode::Enter => {
                if let Err(e) = self.connect_selected() {
                    self.status = Some(StatusMessage::error(format!("Connect failed: {e}")));
                }
            }
            KeyCode::Char('r') => self.scan(),
            KeyCode::Char('t') => {
                self.key_tester.reset();
                self.screen = Screen::KeyTester;
            }
            _ => {}
        }
    }

    fn handle_key_tester_key(&mut self, key: KeyEvent) {
        // Ctrl+R resets all highlights
        if key.code == KeyCode::Char('r') && key.modifiers.contains(KeyModifiers::CONTROL) {
            self.key_tester.reset();
            return;
        }
        let should_exit = self.key_tester.register_key(key.code, key.modifiers);
        if should_exit {
            self.screen = Screen::DeviceSelect;
        }
    }

    fn handle_editor_key(&mut self, key: KeyEvent) {
        // Handle pending g for gg sequence
        if self.pending_g {
            self.pending_g = false;
            if key.code == KeyCode::Char('g') {
                if let Some(keymap) = &mut self.keymap {
                    keymap.jump_col_start(&self.positioned_keys);
                }
                self.count_prefix = None;
                return;
            }
            // Not gg — fall through to handle the key normally
        }

        // Accumulate count prefix digits
        if let KeyCode::Char(c @ '1'..='9') = key.code {
            let digit = c as u32 - '0' as u32;
            self.count_prefix = Some(self.count_prefix.unwrap_or(0) * 10 + digit);
            return;
        }
        if key.code == KeyCode::Char('0') && self.count_prefix.is_some() {
            self.count_prefix = Some(self.count_prefix.unwrap() * 10);
            return;
        }

        let count = self.count_prefix.take().unwrap_or(1) as usize;

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(keymap) = &mut self.keymap {
                    for _ in 0..count {
                        keymap.navigate(Direction::Up, &self.positioned_keys);
                    }
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(keymap) = &mut self.keymap {
                    for _ in 0..count {
                        keymap.navigate(Direction::Down, &self.positioned_keys);
                    }
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(keymap) = &mut self.keymap {
                    for _ in 0..count {
                        keymap.navigate(Direction::Left, &self.positioned_keys);
                    }
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(keymap) = &mut self.keymap {
                    for _ in 0..count {
                        keymap.navigate(Direction::Right, &self.positioned_keys);
                    }
                }
            }
            KeyCode::Char('0') => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.jump_row_start(&self.positioned_keys);
                }
            }
            KeyCode::Char('$') => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.jump_row_end(&self.positioned_keys);
                }
            }
            KeyCode::Char('g') => self.pending_g = true,
            KeyCode::Char('G') => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.jump_col_end(&self.positioned_keys);
                }
            }
            KeyCode::Tab => {
                if let Some(keymap) = &mut self.keymap {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        keymap.prev_layer();
                    } else {
                        keymap.next_layer();
                    }
                }
            }
            KeyCode::BackTab => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.prev_layer();
                }
            }
            KeyCode::Enter => {
                if self.keymap.as_ref().is_some_and(|km| km.selected_key.is_some()) {
                    self.key_picker.open();
                }
            }
            KeyCode::Char('u') => {
                if let Some(keymap) = &mut self.keymap {
                    if keymap.undo().is_some() {
                        self.status = Some(StatusMessage::info("Undo"));
                    }
                }
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some(keymap) = &mut self.keymap {
                    if keymap.redo().is_some() {
                        self.status = Some(StatusMessage::info("Redo"));
                    }
                }
            }
            KeyCode::Char('m') => self.open_macro_editor(),
            KeyCode::Char('L') => self.open_lighting_editor(),
            KeyCode::Char('b') => self.create_backup(),
            KeyCode::Char('r') => self.open_restore_picker(),
            KeyCode::Char('w') => {
                if let Err(e) = self.save_keymap() {
                    self.status = Some(StatusMessage::error(format!("Save failed: {e}")));
                }
            }
            KeyCode::Char('d') => self.disconnect(),
            _ => {}
        }
    }

    fn handle_picker_key(&mut self, key: KeyEvent) {
        use crate::ui::key_picker::PickerMode;

        match self.key_picker.mode {
            PickerMode::Normal => {
                // Handle pending g for gg sequence
                if self.key_picker.pending_g {
                    self.key_picker.pending_g = false;
                    if key.code == KeyCode::Char('g') {
                        self.key_picker.move_top();
                        self.key_picker.count_prefix = None;
                        return;
                    }
                    // Not gg — fall through to handle the key normally
                }

                // Accumulate count prefix digits
                if let KeyCode::Char(c @ '1'..='9') = key.code {
                    let digit = c as u32 - '0' as u32;
                    self.key_picker.count_prefix = Some(
                        self.key_picker.count_prefix.unwrap_or(0) * 10 + digit,
                    );
                    return;
                }
                if key.code == KeyCode::Char('0') && self.key_picker.count_prefix.is_some() {
                    self.key_picker.count_prefix =
                        Some(self.key_picker.count_prefix.unwrap() * 10);
                    return;
                }

                let count = self.key_picker.count_prefix.take().unwrap_or(1) as usize;

                match key.code {
                    KeyCode::Esc => {
                        if self.key_picker.mt_modifier.is_some() {
                            self.key_picker.mt_back();
                        } else {
                            self.key_picker.close();
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        for _ in 0..count {
                            self.key_picker.move_up();
                        }
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        for _ in 0..count {
                            self.key_picker.move_down();
                        }
                    }
                    KeyCode::Left | KeyCode::Char('h') => self.key_picker.prev_category(),
                    KeyCode::Right | KeyCode::Char('l') => self.key_picker.next_category(),
                    KeyCode::Char('/') => self.key_picker.enter_insert(),
                    KeyCode::Char('G') => self.key_picker.move_bottom(),
                    KeyCode::Char('g') => self.key_picker.pending_g = true,
                    KeyCode::Enter => {
                        if let Some(keycode) = self.key_picker.confirm_selection() {
                            if let Some(keymap) = &mut self.keymap {
                                if let Some(key_idx) = keymap.selected_key {
                                    let key = &self.positioned_keys[key_idx];
                                    keymap.set_keycode(key, keycode);
                                }
                            }
                            self.key_picker.close();
                        }
                    }
                    _ => {}
                }
            },
            PickerMode::Insert => match key.code {
                KeyCode::Esc => self.key_picker.enter_normal(),
                KeyCode::Up => self.key_picker.move_up(),
                KeyCode::Down => self.key_picker.move_down(),
                KeyCode::Left => self.key_picker.prev_category(),
                KeyCode::Right => self.key_picker.next_category(),
                KeyCode::Backspace => self.key_picker.backspace(),
                KeyCode::Enter => {
                    if let Some(keycode) = self.key_picker.confirm_selection() {
                        if let Some(keymap) = &mut self.keymap {
                            if let Some(key_idx) = keymap.selected_key {
                                let key = &self.positioned_keys[key_idx];
                                keymap.set_keycode(key, keycode);
                            }
                        }
                        self.key_picker.close();
                    }
                }
                KeyCode::Char(c) => self.key_picker.type_char(c),
                _ => {}
            },
        }
    }

    fn handle_macro_editor_key(&mut self, key: KeyEvent) {
        use crate::keyboard::macros::MacroFocus;

        let Some(state) = &mut self.macro_state else {
            return;
        };

        // Recording mode takes priority — captures all keys except Esc
        if state.recording {
            if key.code == KeyCode::Esc {
                state.stop_recording();
            } else {
                state.record_key(key.code, key.modifiers);
            }
            return;
        }

        match state.focus {
            MacroFocus::List => {
                // Handle pending d for dd sequence
                if self.pending_d {
                    self.pending_d = false;
                    if key.code == KeyCode::Char('d') {
                        if let Some(state) = &mut self.macro_state {
                            state.clear_current();
                        }
                        return;
                    }
                }

                match key.code {
                    KeyCode::Esc => {
                        self.macro_state = None;
                        self.screen = Screen::KeymapEditor;
                    }
                    KeyCode::Up | KeyCode::Char('k') => state.select_up(),
                    KeyCode::Down | KeyCode::Char('j') => state.select_down(),
                    KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => state.focus_editor(),
                    KeyCode::Char('R') => state.start_recording(),
                    KeyCode::Char('d') => self.pending_d = true,
                    KeyCode::Char('w') => self.save_macros(),
                    KeyCode::Char('q') => self.should_quit = true,
                    _ => {}
                }
            }
            MacroFocus::Editor => {
                match key.code {
                    KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left => state.focus_list(),
                    KeyCode::Char('i') | KeyCode::Enter => state.enter_insert(),
                    KeyCode::Char('A') => {
                        state.cursor_pos = state.current_macro().len();
                        state.enter_insert();
                    }
                    KeyCode::Char('I') => {
                        state.cursor_pos = 0;
                        state.enter_insert();
                    }
                    KeyCode::Char('0') => state.cursor_pos = 0,
                    KeyCode::Char('$') => {
                        state.cursor_pos = state.current_macro().len();
                    }
                    KeyCode::Char('w') => self.save_macros(),
                    KeyCode::Char('q') => self.should_quit = true,
                    _ => {}
                }
            }
            MacroFocus::Insert => {
                match key.code {
                    KeyCode::Esc => state.exit_insert(),
                    KeyCode::Backspace => state.backspace(),
                    KeyCode::Left => state.cursor_left(),
                    KeyCode::Right => state.cursor_right(),
                    KeyCode::Char(c) => state.type_char(c),
                    _ => {}
                }
            }
        }
    }

    fn handle_lighting_editor_key(&mut self, key: KeyEvent) {
        let step: i16 = if key.modifiers.contains(KeyModifiers::SHIFT) {
            25
        } else {
            5
        };

        match key.code {
            KeyCode::Esc => {
                self.lighting_state = None;
                self.screen = Screen::KeymapEditor;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(state) = &mut self.lighting_state {
                    state.select_up();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(state) = &mut self.lighting_state {
                    state.select_down();
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(state) = &mut self.lighting_state {
                    state.adjust(step);
                }
                self.apply_lighting_live();
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(state) = &mut self.lighting_state {
                    state.adjust(-step);
                }
                self.apply_lighting_live();
            }
            KeyCode::Char('H') => {
                if let Some(state) = &mut self.lighting_state {
                    state.adjust(-25);
                }
                self.apply_lighting_live();
            }
            KeyCode::Char('L') => {
                if let Some(state) = &mut self.lighting_state {
                    state.adjust(25);
                }
                self.apply_lighting_live();
            }
            KeyCode::Tab => {
                if let Some(state) = &mut self.lighting_state {
                    if key.modifiers.contains(KeyModifiers::SHIFT) {
                        state.prev_section();
                    } else {
                        state.next_section();
                    }
                }
            }
            KeyCode::BackTab => {
                if let Some(state) = &mut self.lighting_state {
                    state.prev_section();
                }
            }
            KeyCode::Char('w') => self.save_lighting_settings(),
            KeyCode::Char('q') => self.should_quit = true,
            _ => {}
        }
    }

    fn handle_backup_picker_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.backup_picker.close(),
            KeyCode::Up | KeyCode::Char('k') => self.backup_picker.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.backup_picker.move_down(),
            KeyCode::Enter => self.restore_selected_backup(),
            _ => {}
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        match self.screen {
            Screen::DeviceSelect => {
                let widget = DeviceSelectorWidget {
                    devices: &self.devices,
                    selected: self.device_selected,
                    scanning: false,
                };
                frame.render_widget(widget, area);
            }
            Screen::KeyTester => {
                let widget = KeyTesterWidget {
                    state: &self.key_tester,
                };
                frame.render_widget(widget, area);
            }
            Screen::MacroEditor => {
                if let Some(state) = &self.macro_state {
                    let name = self
                        .definition
                        .as_ref()
                        .map(|d| d.name.as_str())
                        .unwrap_or("Unknown");
                    let widget = MacroEditorWidget {
                        state,
                        keyboard_name: name,
                    };
                    frame.render_widget(widget, area);
                }
            }
            Screen::LightingEditor => {
                if let Some(state) = &self.lighting_state {
                    let name = self
                        .definition
                        .as_ref()
                        .map(|d| d.name.as_str())
                        .unwrap_or("Unknown");
                    let widget = LightingEditorWidget {
                        state,
                        keyboard_name: name,
                    };
                    frame.render_widget(widget, area);
                }
            }
            Screen::KeymapEditor => {
                if let Some(keymap) = &self.keymap {
                    let name = self
                        .definition
                        .as_ref()
                        .map(|d| d.name.as_str())
                        .unwrap_or("Unknown");

                    let widget = KeymapEditorWidget {
                        keymap,
                        keys: &self.positioned_keys,
                        keyboard_name: name,
                    };
                    frame.render_widget(widget, area);

                    // Key picker overlay
                    if self.key_picker.active {
                        let picker = KeyPickerWidget {
                            state: &self.key_picker,
                        };
                        frame.render_widget(picker, area);
                    }

                    // Backup picker overlay
                    if self.backup_picker.active {
                        let picker = BackupPickerWidget {
                            state: &self.backup_picker,
                        };
                        frame.render_widget(picker, area);
                    }
                }
            }
        }

        // Status bar (bottom line)
        if let Some(status) = &self.status {
            let status_area = ratatui::layout::Rect {
                x: area.x,
                y: area.bottom().saturating_sub(1),
                width: area.width,
                height: 1,
            };
            let widget = StatusBarWidget {
                message: Some(status),
            };
            frame.render_widget(widget, status_area);
        }
    }
}
