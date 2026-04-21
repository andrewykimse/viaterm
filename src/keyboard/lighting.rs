use anyhow::Result;

use crate::keyboard::connection::KeyboardConnection;

/// Which lighting subsystems the keyboard supports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightingType {
    Backlight,
    RgbLight,
    RgbMatrix,
    LedMatrix,
}

impl LightingType {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Backlight => "Backlight",
            Self::RgbLight => "RGB Light",
            Self::RgbMatrix => "RGB Matrix",
            Self::LedMatrix => "LED Matrix",
        }
    }
}

/// A single adjustable lighting parameter.
#[derive(Debug, Clone)]
pub struct LightingParam {
    pub name: &'static str,
    pub value: u8,
    pub max: u8,
}

impl LightingParam {
    fn new(name: &'static str, value: u8) -> Self {
        Self {
            name,
            value,
            max: 255,
        }
    }
}

/// A detected lighting subsystem with its current parameter values.
#[derive(Debug, Clone)]
pub struct LightingSection {
    pub lighting_type: LightingType,
    pub params: Vec<LightingParam>,
}

/// Full lighting state for the editor.
pub struct LightingState {
    pub sections: Vec<LightingSection>,
    pub active_section: usize,
    pub selected_param: usize,
    pub dirty: bool,
}

impl LightingState {
    pub fn current_section(&self) -> Option<&LightingSection> {
        self.sections.get(self.active_section)
    }

    pub fn param_count(&self) -> usize {
        self.current_section().map_or(0, |s| s.params.len())
    }

    pub fn next_section(&mut self) {
        if !self.sections.is_empty() {
            self.active_section = (self.active_section + 1) % self.sections.len();
            self.selected_param = 0;
        }
    }

    pub fn prev_section(&mut self) {
        if !self.sections.is_empty() {
            self.active_section = (self.active_section + self.sections.len() - 1)
                % self.sections.len();
            self.selected_param = 0;
        }
    }

    pub fn select_up(&mut self) {
        if self.selected_param > 0 {
            self.selected_param -= 1;
        }
    }

    pub fn select_down(&mut self) {
        let count = self.param_count();
        if count > 0 && self.selected_param + 1 < count {
            self.selected_param += 1;
        }
    }

    /// Adjust the selected parameter by `delta` (positive = increase).
    pub fn adjust(&mut self, delta: i16) {
        if let Some(section) = self.sections.get_mut(self.active_section) {
            if let Some(param) = section.params.get_mut(self.selected_param) {
                let new_val = (param.value as i16 + delta).clamp(0, param.max as i16) as u8;
                if new_val != param.value {
                    param.value = new_val;
                    self.dirty = true;
                }
            }
        }
    }
}

/// Probe the keyboard to detect supported lighting types and read current values.
pub fn detect_lighting(conn: &KeyboardConnection) -> Result<LightingState> {
    let api = &conn.api;
    let mut sections = Vec::new();

    // Probe backlight
    if let Ok(brightness) = api.get_backlight_brightness() {
        let effect = api.get_backlight_effect().unwrap_or(0);
        sections.push(LightingSection {
            lighting_type: LightingType::Backlight,
            params: vec![
                LightingParam::new("Brightness", brightness),
                LightingParam::new("Effect", effect),
            ],
        });
    }

    // Probe RGB Light (underglow)
    if let Ok(brightness) = api.get_rgblight_brightness() {
        let effect = api.get_rgblight_effect().unwrap_or(0);
        let speed = api.get_rgblight_effect_speed().unwrap_or(0);
        let (hue, sat) = api.get_rgblight_color().unwrap_or((0, 0));
        sections.push(LightingSection {
            lighting_type: LightingType::RgbLight,
            params: vec![
                LightingParam::new("Brightness", brightness),
                LightingParam::new("Effect", effect),
                LightingParam::new("Speed", speed),
                LightingParam::new("Hue", hue),
                LightingParam::new("Saturation", sat),
            ],
        });
    }

    // Probe RGB Matrix (per-key, protocol v3+)
    if let Ok(brightness) = api.get_rgb_matrix_brightness() {
        let effect = api.get_rgb_matrix_effect().unwrap_or(0);
        let speed = api.get_rgb_matrix_effect_speed().unwrap_or(0);
        let (hue, sat) = api.get_rgb_matrix_color().unwrap_or((0, 0));
        sections.push(LightingSection {
            lighting_type: LightingType::RgbMatrix,
            params: vec![
                LightingParam::new("Brightness", brightness),
                LightingParam::new("Effect", effect),
                LightingParam::new("Speed", speed),
                LightingParam::new("Hue", hue),
                LightingParam::new("Saturation", sat),
            ],
        });
    }

    // Probe LED Matrix (monochrome per-key, protocol v3+)
    if let Ok(brightness) = api.get_led_matrix_brightness() {
        let effect = api.get_led_matrix_effect().unwrap_or(0);
        let speed = api.get_led_matrix_effect_speed().unwrap_or(0);
        sections.push(LightingSection {
            lighting_type: LightingType::LedMatrix,
            params: vec![
                LightingParam::new("Brightness", brightness),
                LightingParam::new("Effect", effect),
                LightingParam::new("Speed", speed),
            ],
        });
    }

    Ok(LightingState {
        sections,
        active_section: 0,
        selected_param: 0,
        dirty: false,
    })
}

/// Push the current lighting values to the keyboard (live preview, not persisted).
pub fn apply_lighting(conn: &KeyboardConnection, state: &LightingState) -> Result<()> {
    let api = &conn.api;

    for section in &state.sections {
        let vals: Vec<u8> = section.params.iter().map(|p| p.value).collect();
        match section.lighting_type {
            LightingType::Backlight => {
                let _ = api.set_backlight_brightness(vals[0]);
                let _ = api.set_backlight_effect(vals[1]);
            }
            LightingType::RgbLight => {
                let _ = api.set_rgblight_brightness(vals[0]);
                let _ = api.set_rgblight_effect(vals[1]);
                let _ = api.set_rgblight_effect_speed(vals[2]);
                let _ = api.set_rgblight_color(vals[3], vals[4]);
            }
            LightingType::RgbMatrix => {
                let _ = api.set_rgb_matrix_brightness(vals[0]);
                let _ = api.set_rgb_matrix_effect(vals[1]);
                let _ = api.set_rgb_matrix_effect_speed(vals[2]);
                let _ = api.set_rgb_matrix_color(vals[3], vals[4]);
            }
            LightingType::LedMatrix => {
                let _ = api.set_led_matrix_brightness(vals[0]);
                let _ = api.set_led_matrix_effect(vals[1]);
                let _ = api.set_led_matrix_effect_speed(vals[2]);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rgb_section() -> LightingSection {
        LightingSection {
            lighting_type: LightingType::RgbLight,
            params: vec![
                LightingParam::new("Brightness", 128),
                LightingParam::new("Effect", 5),
                LightingParam::new("Speed", 100),
                LightingParam::new("Hue", 200),
                LightingParam::new("Saturation", 255),
            ],
        }
    }

    fn make_backlight_section() -> LightingSection {
        LightingSection {
            lighting_type: LightingType::Backlight,
            params: vec![
                LightingParam::new("Brightness", 64),
                LightingParam::new("Effect", 1),
            ],
        }
    }

    fn make_state() -> LightingState {
        LightingState {
            sections: vec![make_backlight_section(), make_rgb_section()],
            active_section: 0,
            selected_param: 0,
            dirty: false,
        }
    }

    // --- LightingType ---

    #[test]
    fn lighting_type_labels() {
        assert_eq!(LightingType::Backlight.label(), "Backlight");
        assert_eq!(LightingType::RgbLight.label(), "RGB Light");
        assert_eq!(LightingType::RgbMatrix.label(), "RGB Matrix");
        assert_eq!(LightingType::LedMatrix.label(), "LED Matrix");
    }

    // --- LightingParam ---

    #[test]
    fn param_default_max() {
        let p = LightingParam::new("Test", 42);
        assert_eq!(p.value, 42);
        assert_eq!(p.max, 255);
    }

    // --- section navigation ---

    #[test]
    fn next_section_wraps() {
        let mut s = make_state();
        assert_eq!(s.active_section, 0);
        s.next_section();
        assert_eq!(s.active_section, 1);
        s.next_section();
        assert_eq!(s.active_section, 0); // wraps
    }

    #[test]
    fn prev_section_wraps() {
        let mut s = make_state();
        s.prev_section();
        assert_eq!(s.active_section, 1); // wraps to last
        s.prev_section();
        assert_eq!(s.active_section, 0);
    }

    #[test]
    fn section_switch_resets_param_selection() {
        let mut s = make_state();
        s.selected_param = 1;
        s.next_section();
        assert_eq!(s.selected_param, 0);
    }

    // --- param navigation ---

    #[test]
    fn select_down_up() {
        let mut s = make_state();
        s.select_down();
        assert_eq!(s.selected_param, 1);
        s.select_up();
        assert_eq!(s.selected_param, 0);
    }

    #[test]
    fn select_down_clamps() {
        let mut s = make_state();
        // Backlight section has 2 params (indices 0,1)
        s.select_down();
        s.select_down(); // should clamp at 1
        assert_eq!(s.selected_param, 1);
    }

    #[test]
    fn select_up_clamps_at_zero() {
        let mut s = make_state();
        s.select_up(); // already at 0
        assert_eq!(s.selected_param, 0);
    }

    // --- adjust ---

    #[test]
    fn adjust_increases() {
        let mut s = make_state();
        let before = s.sections[0].params[0].value; // 64
        s.adjust(10);
        assert_eq!(s.sections[0].params[0].value, before + 10);
        assert!(s.dirty);
    }

    #[test]
    fn adjust_decreases() {
        let mut s = make_state();
        let before = s.sections[0].params[0].value;
        s.adjust(-10);
        assert_eq!(s.sections[0].params[0].value, before - 10);
    }

    #[test]
    fn adjust_clamps_at_max() {
        let mut s = make_state();
        s.adjust(255); // way over max
        assert_eq!(s.sections[0].params[0].value, 255);
    }

    #[test]
    fn adjust_clamps_at_zero() {
        let mut s = make_state();
        s.adjust(-255); // way under min
        assert_eq!(s.sections[0].params[0].value, 0);
    }

    #[test]
    fn adjust_no_change_not_dirty() {
        let mut s = make_state();
        s.sections[0].params[0].value = 0;
        s.adjust(-5); // already at 0, clamped to 0
        assert!(!s.dirty);
    }

    #[test]
    fn adjust_targets_selected_param() {
        let mut s = make_state();
        s.next_section(); // switch to RGB section
        s.selected_param = 3; // Hue
        let before = s.sections[1].params[3].value; // 200
        s.adjust(5);
        assert_eq!(s.sections[1].params[3].value, before + 5);
    }

    // --- current_section / param_count ---

    #[test]
    fn current_section() {
        let s = make_state();
        let section = s.current_section().unwrap();
        assert_eq!(section.lighting_type, LightingType::Backlight);
    }

    #[test]
    fn param_count() {
        let s = make_state();
        assert_eq!(s.param_count(), 2); // backlight has 2 params
    }

    #[test]
    fn empty_state() {
        let s = LightingState {
            sections: vec![],
            active_section: 0,
            selected_param: 0,
            dirty: false,
        };
        assert!(s.current_section().is_none());
        assert_eq!(s.param_count(), 0);
    }
}

/// Persist lighting settings to keyboard EEPROM.
pub fn save_lighting(conn: &KeyboardConnection) -> Result<()> {
    conn.api
        .save_lighting()
        .map_err(|e| anyhow::anyhow!("Failed to save lighting: {e:?}"))
}
