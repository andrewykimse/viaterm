use std::path::PathBuf;

use anyhow::{Context, Result};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use qmk_via_api::scan::KeyboardDeviceInfo;
use ratatui::Frame;

use crate::definition::layout_parser::{PositionedKey, parse_layout};
use crate::definition::loader::{fetch_definition, load_definition_file};
use crate::definition::schema::ViaDefinition;
use crate::event::is_quit;
use crate::keyboard::connection::{KeyboardConnection, scan_devices};
use crate::keyboard::keymap::KeymapState;
use crate::keyboard::keymap::Direction;
use crate::ui::device_selector::DeviceSelectorWidget;
use crate::ui::key_picker::{KeyPickerState, KeyPickerWidget};
use crate::ui::key_tester::{KeyTesterState, KeyTesterWidget};
use crate::ui::keymap_editor::KeymapEditorWidget;
use crate::ui::status_bar::{StatusBarWidget, StatusMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    DeviceSelect,
    KeymapEditor,
    KeyTester,
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

    // Key picker
    pub key_picker: KeyPickerState,

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
            key_picker: KeyPickerState::new(),
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

    pub fn handle_event(&mut self, event: Event) {
        if let Event::Key(key) = event {
            if self.key_picker.active {
                self.handle_picker_key(key);
            } else {
                match self.screen {
                    Screen::DeviceSelect => self.handle_device_select_key(key),
                    Screen::KeymapEditor => self.handle_editor_key(key),
                    Screen::KeyTester => self.handle_key_tester_key(key),
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
                return;
            }
            // Not gg — fall through to handle the key normally
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.navigate(Direction::Up, &self.positioned_keys);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.navigate(Direction::Down, &self.positioned_keys);
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.navigate(Direction::Left, &self.positioned_keys);
                }
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(keymap) = &mut self.keymap {
                    keymap.navigate(Direction::Right, &self.positioned_keys);
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
                        return;
                    }
                    // Not gg — fall through to handle the key normally
                }

                match key.code {
                    KeyCode::Esc => self.key_picker.close(),
                    KeyCode::Up | KeyCode::Char('k') => self.key_picker.move_up(),
                    KeyCode::Down | KeyCode::Char('j') => self.key_picker.move_down(),
                    KeyCode::Left | KeyCode::Char('h') => self.key_picker.prev_category(),
                    KeyCode::Right | KeyCode::Char('l') => self.key_picker.next_category(),
                    KeyCode::Char('/') => self.key_picker.enter_insert(),
                    KeyCode::Char('G') => self.key_picker.move_bottom(),
                    KeyCode::Char('g') => self.key_picker.pending_g = true,
                    KeyCode::Enter => {
                        if let Some(keycode) = self.key_picker.selected_keycode() {
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
                    if let Some(keycode) = self.key_picker.selected_keycode() {
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
