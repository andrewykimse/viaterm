/// Categories for organizing keycodes in the picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeycodeCategory {
    Basic,
    Modifiers,
    Navigation,
    FunctionKeys,
    Media,
    NumpadKeys,
    Macros,
    LayerFunctions,
    ModTap,
    Special,
}

impl KeycodeCategory {
    pub const ALL: &[Self] = &[
        Self::Basic,
        Self::Modifiers,
        Self::Navigation,
        Self::FunctionKeys,
        Self::Media,
        Self::NumpadKeys,
        Self::Macros,
        Self::LayerFunctions,
        Self::ModTap,
        Self::Special,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Basic => "Basic",
            Self::Modifiers => "Modifiers",
            Self::Navigation => "Navigation",
            Self::FunctionKeys => "Function",
            Self::Media => "Media",
            Self::NumpadKeys => "Numpad",
            Self::Macros => "Macros",
            Self::LayerFunctions => "Layers",
            Self::ModTap => "Mod-Tap",
            Self::Special => "Special",
        }
    }
}

/// A keycode entry for the picker.
#[derive(Debug, Clone)]
pub struct KeycodeEntry {
    pub code: u16,
    pub name: &'static str,
    pub label: &'static str,
    pub category: KeycodeCategory,
}

/// Get the display label for a keycode (short, for rendering on keys).
pub fn keycode_label(code: u16) -> String {
    if let Some(entry) = KEYCODES.iter().find(|e| e.code == code) {
        return entry.label.to_string();
    }

    // QMK keycode ranges
    match code {
        0x0000 => "____".to_string(),
        0x0001 => "TRNS".to_string(),

        // Layer tap: LT(layer, kc) = 0x4000 | (layer << 8) | kc
        0x4000..=0x4FFF => {
            let layer = (code >> 8) & 0x0F;
            let kc = code & 0xFF;
            let base = keycode_label(kc);
            format!("LT{layer}({base})")
        }

        // MO(layer)
        0x5100..=0x511F => format!("MO({})", code & 0x1F),
        // TG(layer)
        0x5140..=0x515F => format!("TG({})", code & 0x1F),
        // TT(layer)
        0x5180..=0x519F => format!("TT({})", code & 0x1F),
        // OSL(layer)
        0x5160..=0x517F => format!("OSL({})", code & 0x1F),
        // TO(layer)
        0x5200..=0x521F => format!("TO({})", code & 0x1F),

        // Mod-tap: MT(mod, kc) = 0x6000+ range
        0x6000..=0x76FF => {
            let kc = code & 0xFF;
            let mods = (code >> 8) & 0x1F;
            let base = keycode_label(kc);
            let mod_str = mod_bits_to_string(mods);
            format!("MT({mod_str},{base})")
        }

        // Macro range
        0x7700..=0x77FF => format!("M{}", code - 0x7700),

        // Remaining mod-tap range
        0x7800..=0x7FFF => {
            let kc = code & 0xFF;
            let mods = (code >> 8) & 0x1F;
            let base = keycode_label(kc);
            let mod_str = mod_bits_to_string(mods);
            format!("MT({mod_str},{base})")
        }

        _ => format!("{code:#06X}"),
    }
}

/// Modifier options for the MT picker (first step).
pub static MT_MODIFIERS: &[KeycodeEntry] = &[
    KeycodeEntry { code: 0x01, name: "Ctrl", label: "CTL", category: ModTap },
    KeycodeEntry { code: 0x02, name: "Shift", label: "SFT", category: ModTap },
    KeycodeEntry { code: 0x04, name: "Alt", label: "ALT", category: ModTap },
    KeycodeEntry { code: 0x08, name: "GUI", label: "GUI", category: ModTap },
    KeycodeEntry { code: 0x03, name: "Ctrl+Shift", label: "CS", category: ModTap },
    KeycodeEntry { code: 0x05, name: "Ctrl+Alt", label: "CA", category: ModTap },
    KeycodeEntry { code: 0x09, name: "Ctrl+GUI", label: "CG", category: ModTap },
    KeycodeEntry { code: 0x06, name: "Shift+Alt", label: "SA", category: ModTap },
    KeycodeEntry { code: 0x0A, name: "Shift+GUI", label: "SG", category: ModTap },
    KeycodeEntry { code: 0x0C, name: "Alt+GUI", label: "AG", category: ModTap },
];

/// Get base keycodes available for MT tap behavior.
pub fn mt_base_keycodes() -> Vec<&'static KeycodeEntry> {
    KEYCODES
        .iter()
        .filter(|e| e.code <= 0xFF && e.category != KeycodeCategory::Special)
        .collect()
}

/// Encode an MT keycode from modifier bits and base keycode.
pub fn encode_mt(mod_bits: u16, base_kc: u16) -> u16 {
    0x6000 | (mod_bits << 8) | (base_kc & 0xFF)
}

fn mod_bits_to_string(mods: u16) -> &'static str {
    match mods {
        0x01 => "CTL",
        0x02 => "SFT",
        0x04 => "ALT",
        0x08 => "GUI",
        0x03 => "CS",
        0x05 => "CA",
        0x09 => "CG",
        0x06 => "SA",
        0x0A => "SG",
        _ => "MOD",
    }
}

/// Get keycodes filtered by category and optional search query.
pub fn filtered_keycodes(
    category: KeycodeCategory,
    query: Option<&str>,
) -> Vec<&'static KeycodeEntry> {
    KEYCODES
        .iter()
        .filter(|e| e.category == category)
        .filter(|e| {
            query.is_none_or(|q| {
                let q = q.to_lowercase();
                e.name.to_lowercase().contains(&q) || e.label.to_lowercase().contains(&q)
            })
        })
        .collect()
}

/// All searchable keycodes.
pub fn search_keycodes(query: &str) -> Vec<&'static KeycodeEntry> {
    let q = query.to_lowercase();
    KEYCODES
        .iter()
        .filter(|e| e.name.to_lowercase().contains(&q) || e.label.to_lowercase().contains(&q))
        .collect()
}

use KeycodeCategory::*;

static KEYCODES: &[KeycodeEntry] = &[
    // Basic keys
    KeycodeEntry { code: 0x04, name: "A", label: "A", category: Basic },
    KeycodeEntry { code: 0x05, name: "B", label: "B", category: Basic },
    KeycodeEntry { code: 0x06, name: "C", label: "C", category: Basic },
    KeycodeEntry { code: 0x07, name: "D", label: "D", category: Basic },
    KeycodeEntry { code: 0x08, name: "E", label: "E", category: Basic },
    KeycodeEntry { code: 0x09, name: "F", label: "F", category: Basic },
    KeycodeEntry { code: 0x0A, name: "G", label: "G", category: Basic },
    KeycodeEntry { code: 0x0B, name: "H", label: "H", category: Basic },
    KeycodeEntry { code: 0x0C, name: "I", label: "I", category: Basic },
    KeycodeEntry { code: 0x0D, name: "J", label: "J", category: Basic },
    KeycodeEntry { code: 0x0E, name: "K", label: "K", category: Basic },
    KeycodeEntry { code: 0x0F, name: "L", label: "L", category: Basic },
    KeycodeEntry { code: 0x10, name: "M", label: "M", category: Basic },
    KeycodeEntry { code: 0x11, name: "N", label: "N", category: Basic },
    KeycodeEntry { code: 0x12, name: "O", label: "O", category: Basic },
    KeycodeEntry { code: 0x13, name: "P", label: "P", category: Basic },
    KeycodeEntry { code: 0x14, name: "Q", label: "Q", category: Basic },
    KeycodeEntry { code: 0x15, name: "R", label: "R", category: Basic },
    KeycodeEntry { code: 0x16, name: "S", label: "S", category: Basic },
    KeycodeEntry { code: 0x17, name: "T", label: "T", category: Basic },
    KeycodeEntry { code: 0x18, name: "U", label: "U", category: Basic },
    KeycodeEntry { code: 0x19, name: "V", label: "V", category: Basic },
    KeycodeEntry { code: 0x1A, name: "W", label: "W", category: Basic },
    KeycodeEntry { code: 0x1B, name: "X", label: "X", category: Basic },
    KeycodeEntry { code: 0x1C, name: "Y", label: "Y", category: Basic },
    KeycodeEntry { code: 0x1D, name: "Z", label: "Z", category: Basic },
    KeycodeEntry { code: 0x1E, name: "1", label: "1", category: Basic },
    KeycodeEntry { code: 0x1F, name: "2", label: "2", category: Basic },
    KeycodeEntry { code: 0x20, name: "3", label: "3", category: Basic },
    KeycodeEntry { code: 0x21, name: "4", label: "4", category: Basic },
    KeycodeEntry { code: 0x22, name: "5", label: "5", category: Basic },
    KeycodeEntry { code: 0x23, name: "6", label: "6", category: Basic },
    KeycodeEntry { code: 0x24, name: "7", label: "7", category: Basic },
    KeycodeEntry { code: 0x25, name: "8", label: "8", category: Basic },
    KeycodeEntry { code: 0x26, name: "9", label: "9", category: Basic },
    KeycodeEntry { code: 0x27, name: "0", label: "0", category: Basic },
    KeycodeEntry { code: 0x28, name: "Enter", label: "ENT", category: Basic },
    KeycodeEntry { code: 0x29, name: "Escape", label: "ESC", category: Basic },
    KeycodeEntry { code: 0x2A, name: "Backspace", label: "BSPC", category: Basic },
    KeycodeEntry { code: 0x2B, name: "Tab", label: "TAB", category: Basic },
    KeycodeEntry { code: 0x2C, name: "Space", label: "SPC", category: Basic },
    KeycodeEntry { code: 0x2D, name: "Minus", label: "-", category: Basic },
    KeycodeEntry { code: 0x2E, name: "Equal", label: "=", category: Basic },
    KeycodeEntry { code: 0x2F, name: "Left Bracket", label: "[", category: Basic },
    KeycodeEntry { code: 0x30, name: "Right Bracket", label: "]", category: Basic },
    KeycodeEntry { code: 0x31, name: "Backslash", label: "\\", category: Basic },
    KeycodeEntry { code: 0x33, name: "Semicolon", label: ";", category: Basic },
    KeycodeEntry { code: 0x34, name: "Quote", label: "'", category: Basic },
    KeycodeEntry { code: 0x35, name: "Grave", label: "`", category: Basic },
    KeycodeEntry { code: 0x36, name: "Comma", label: ",", category: Basic },
    KeycodeEntry { code: 0x37, name: "Period", label: ".", category: Basic },
    KeycodeEntry { code: 0x38, name: "Slash", label: "/", category: Basic },
    KeycodeEntry { code: 0x39, name: "Caps Lock", label: "CAPS", category: Basic },

    // Modifiers
    KeycodeEntry { code: 0xE0, name: "Left Ctrl", label: "LCTL", category: Modifiers },
    KeycodeEntry { code: 0xE1, name: "Left Shift", label: "LSFT", category: Modifiers },
    KeycodeEntry { code: 0xE2, name: "Left Alt", label: "LALT", category: Modifiers },
    KeycodeEntry { code: 0xE3, name: "Left GUI", label: "LGUI", category: Modifiers },
    KeycodeEntry { code: 0xE4, name: "Right Ctrl", label: "RCTL", category: Modifiers },
    KeycodeEntry { code: 0xE5, name: "Right Shift", label: "RSFT", category: Modifiers },
    KeycodeEntry { code: 0xE6, name: "Right Alt", label: "RALT", category: Modifiers },
    KeycodeEntry { code: 0xE7, name: "Right GUI", label: "RGUI", category: Modifiers },

    // Navigation
    KeycodeEntry { code: 0x4F, name: "Right", label: "→", category: Navigation },
    KeycodeEntry { code: 0x50, name: "Left", label: "←", category: Navigation },
    KeycodeEntry { code: 0x51, name: "Down", label: "↓", category: Navigation },
    KeycodeEntry { code: 0x52, name: "Up", label: "↑", category: Navigation },
    KeycodeEntry { code: 0x4A, name: "Home", label: "HOME", category: Navigation },
    KeycodeEntry { code: 0x4D, name: "End", label: "END", category: Navigation },
    KeycodeEntry { code: 0x4B, name: "Page Up", label: "PGUP", category: Navigation },
    KeycodeEntry { code: 0x4E, name: "Page Down", label: "PGDN", category: Navigation },
    KeycodeEntry { code: 0x49, name: "Insert", label: "INS", category: Navigation },
    KeycodeEntry { code: 0x4C, name: "Delete", label: "DEL", category: Navigation },
    KeycodeEntry { code: 0x46, name: "Print Screen", label: "PSCR", category: Navigation },
    KeycodeEntry { code: 0x47, name: "Scroll Lock", label: "SLCK", category: Navigation },
    KeycodeEntry { code: 0x48, name: "Pause", label: "PAUS", category: Navigation },

    // Function keys
    KeycodeEntry { code: 0x3A, name: "F1", label: "F1", category: FunctionKeys },
    KeycodeEntry { code: 0x3B, name: "F2", label: "F2", category: FunctionKeys },
    KeycodeEntry { code: 0x3C, name: "F3", label: "F3", category: FunctionKeys },
    KeycodeEntry { code: 0x3D, name: "F4", label: "F4", category: FunctionKeys },
    KeycodeEntry { code: 0x3E, name: "F5", label: "F5", category: FunctionKeys },
    KeycodeEntry { code: 0x3F, name: "F6", label: "F6", category: FunctionKeys },
    KeycodeEntry { code: 0x40, name: "F7", label: "F7", category: FunctionKeys },
    KeycodeEntry { code: 0x41, name: "F8", label: "F8", category: FunctionKeys },
    KeycodeEntry { code: 0x42, name: "F9", label: "F9", category: FunctionKeys },
    KeycodeEntry { code: 0x43, name: "F10", label: "F10", category: FunctionKeys },
    KeycodeEntry { code: 0x44, name: "F11", label: "F11", category: FunctionKeys },
    KeycodeEntry { code: 0x45, name: "F12", label: "F12", category: FunctionKeys },
    KeycodeEntry { code: 0x68, name: "F13", label: "F13", category: FunctionKeys },
    KeycodeEntry { code: 0x69, name: "F14", label: "F14", category: FunctionKeys },
    KeycodeEntry { code: 0x6A, name: "F15", label: "F15", category: FunctionKeys },
    KeycodeEntry { code: 0x6B, name: "F16", label: "F16", category: FunctionKeys },
    KeycodeEntry { code: 0x6C, name: "F17", label: "F17", category: FunctionKeys },
    KeycodeEntry { code: 0x6D, name: "F18", label: "F18", category: FunctionKeys },
    KeycodeEntry { code: 0x6E, name: "F19", label: "F19", category: FunctionKeys },
    KeycodeEntry { code: 0x6F, name: "F20", label: "F20", category: FunctionKeys },
    KeycodeEntry { code: 0x70, name: "F21", label: "F21", category: FunctionKeys },
    KeycodeEntry { code: 0x71, name: "F22", label: "F22", category: FunctionKeys },
    KeycodeEntry { code: 0x72, name: "F23", label: "F23", category: FunctionKeys },
    KeycodeEntry { code: 0x73, name: "F24", label: "F24", category: FunctionKeys },

    // Media
    KeycodeEntry { code: 0x00A8, name: "Volume Up", label: "VOLU", category: Media },
    KeycodeEntry { code: 0x00A9, name: "Volume Down", label: "VOLD", category: Media },
    KeycodeEntry { code: 0x00A7, name: "Mute", label: "MUTE", category: Media },
    KeycodeEntry { code: 0x00A5, name: "Play/Pause", label: "MPLY", category: Media },
    KeycodeEntry { code: 0x00A6, name: "Next Track", label: "MNXT", category: Media },
    KeycodeEntry { code: 0x00A4, name: "Prev Track", label: "MPRV", category: Media },
    KeycodeEntry { code: 0x00AA, name: "Media Stop", label: "MSTP", category: Media },

    // Numpad
    KeycodeEntry { code: 0x53, name: "Num Lock", label: "NLCK", category: NumpadKeys },
    KeycodeEntry { code: 0x54, name: "Numpad /", label: "P/", category: NumpadKeys },
    KeycodeEntry { code: 0x55, name: "Numpad *", label: "P*", category: NumpadKeys },
    KeycodeEntry { code: 0x56, name: "Numpad -", label: "P-", category: NumpadKeys },
    KeycodeEntry { code: 0x57, name: "Numpad +", label: "P+", category: NumpadKeys },
    KeycodeEntry { code: 0x58, name: "Numpad Enter", label: "PENT", category: NumpadKeys },
    KeycodeEntry { code: 0x59, name: "Numpad 1", label: "P1", category: NumpadKeys },
    KeycodeEntry { code: 0x5A, name: "Numpad 2", label: "P2", category: NumpadKeys },
    KeycodeEntry { code: 0x5B, name: "Numpad 3", label: "P3", category: NumpadKeys },
    KeycodeEntry { code: 0x5C, name: "Numpad 4", label: "P4", category: NumpadKeys },
    KeycodeEntry { code: 0x5D, name: "Numpad 5", label: "P5", category: NumpadKeys },
    KeycodeEntry { code: 0x5E, name: "Numpad 6", label: "P6", category: NumpadKeys },
    KeycodeEntry { code: 0x5F, name: "Numpad 7", label: "P7", category: NumpadKeys },
    KeycodeEntry { code: 0x60, name: "Numpad 8", label: "P8", category: NumpadKeys },
    KeycodeEntry { code: 0x61, name: "Numpad 9", label: "P9", category: NumpadKeys },
    KeycodeEntry { code: 0x62, name: "Numpad 0", label: "P0", category: NumpadKeys },
    KeycodeEntry { code: 0x63, name: "Numpad .", label: "P.", category: NumpadKeys },

    // Macros
    KeycodeEntry { code: 0x7700, name: "Macro 0", label: "M0", category: Macros },
    KeycodeEntry { code: 0x7701, name: "Macro 1", label: "M1", category: Macros },
    KeycodeEntry { code: 0x7702, name: "Macro 2", label: "M2", category: Macros },
    KeycodeEntry { code: 0x7703, name: "Macro 3", label: "M3", category: Macros },
    KeycodeEntry { code: 0x7704, name: "Macro 4", label: "M4", category: Macros },
    KeycodeEntry { code: 0x7705, name: "Macro 5", label: "M5", category: Macros },
    KeycodeEntry { code: 0x7706, name: "Macro 6", label: "M6", category: Macros },
    KeycodeEntry { code: 0x7707, name: "Macro 7", label: "M7", category: Macros },
    KeycodeEntry { code: 0x7708, name: "Macro 8", label: "M8", category: Macros },
    KeycodeEntry { code: 0x7709, name: "Macro 9", label: "M9", category: Macros },
    KeycodeEntry { code: 0x770A, name: "Macro 10", label: "M10", category: Macros },
    KeycodeEntry { code: 0x770B, name: "Macro 11", label: "M11", category: Macros },
    KeycodeEntry { code: 0x770C, name: "Macro 12", label: "M12", category: Macros },
    KeycodeEntry { code: 0x770D, name: "Macro 13", label: "M13", category: Macros },
    KeycodeEntry { code: 0x770E, name: "Macro 14", label: "M14", category: Macros },
    KeycodeEntry { code: 0x770F, name: "Macro 15", label: "M15", category: Macros },

    // Layer functions — codes must match QMK's quantum keycode ranges
    KeycodeEntry { code: 0x5100, name: "MO(0)", label: "MO(0)", category: LayerFunctions },
    KeycodeEntry { code: 0x5101, name: "MO(1)", label: "MO(1)", category: LayerFunctions },
    KeycodeEntry { code: 0x5102, name: "MO(2)", label: "MO(2)", category: LayerFunctions },
    KeycodeEntry { code: 0x5103, name: "MO(3)", label: "MO(3)", category: LayerFunctions },
    KeycodeEntry { code: 0x5140, name: "TG(0)", label: "TG(0)", category: LayerFunctions },
    KeycodeEntry { code: 0x5141, name: "TG(1)", label: "TG(1)", category: LayerFunctions },
    KeycodeEntry { code: 0x5142, name: "TG(2)", label: "TG(2)", category: LayerFunctions },
    KeycodeEntry { code: 0x5143, name: "TG(3)", label: "TG(3)", category: LayerFunctions },
    KeycodeEntry { code: 0x5200, name: "TO(0)", label: "TO(0)", category: LayerFunctions },
    KeycodeEntry { code: 0x5201, name: "TO(1)", label: "TO(1)", category: LayerFunctions },
    KeycodeEntry { code: 0x5202, name: "TO(2)", label: "TO(2)", category: LayerFunctions },
    KeycodeEntry { code: 0x5203, name: "TO(3)", label: "TO(3)", category: LayerFunctions },

    // Special
    KeycodeEntry { code: 0x0000, name: "None", label: "____", category: Special },
    KeycodeEntry { code: 0x0001, name: "Transparent", label: "TRNS", category: Special },
    KeycodeEntry { code: 0x5C00, name: "Reset", label: "RST", category: Special },
    KeycodeEntry { code: 0x5C01, name: "Debug", label: "DBG", category: Special },
    KeycodeEntry { code: 0x5C10, name: "Toggle NKRO", label: "NKRO", category: Special },
];

#[cfg(test)]
mod tests {
    use super::*;

    // --- keycode_label ---

    #[test]
    fn label_basic_key() {
        assert_eq!(keycode_label(0x04), "A");
        assert_eq!(keycode_label(0x28), "ENT");
    }

    #[test]
    fn label_none_and_transparent() {
        assert_eq!(keycode_label(0x0000), "____");
        assert_eq!(keycode_label(0x0001), "TRNS");
    }

    #[test]
    fn label_modifier() {
        assert_eq!(keycode_label(0xE0), "LCTL");
        assert_eq!(keycode_label(0xE7), "RGUI");
    }

    #[test]
    fn label_layer_tap() {
        // LT(1, KC_A) = 0x4000 | (1 << 8) | 0x04
        assert_eq!(keycode_label(0x4104), "LT1(A)");
    }

    #[test]
    fn label_mo() {
        assert_eq!(keycode_label(0x5100), "MO(0)");
        assert_eq!(keycode_label(0x5103), "MO(3)");
    }

    #[test]
    fn label_tg_to_tt_osl() {
        assert_eq!(keycode_label(0x5141), "TG(1)");
        assert_eq!(keycode_label(0x5202), "TO(2)");
        assert_eq!(keycode_label(0x5180), "TT(0)");
        assert_eq!(keycode_label(0x5163), "OSL(3)");
    }

    #[test]
    fn label_mod_tap() {
        // MT(CTL, A) = 0x6000 | (0x01 << 8) | 0x04
        assert_eq!(keycode_label(0x6104), "MT(CTL,A)");
        // MT(SFT, ENT) = 0x6000 | (0x02 << 8) | 0x28
        assert_eq!(keycode_label(0x6228), "MT(SFT,ENT)");
    }

    #[test]
    fn label_macro_range() {
        assert_eq!(keycode_label(0x7700), "M0");
        assert_eq!(keycode_label(0x7705), "M5");
        assert_eq!(keycode_label(0x77FF), "M255");
    }

    #[test]
    fn label_unknown_hex() {
        // A code not in any known range
        assert_eq!(keycode_label(0xFFFF), "0xFFFF");
    }

    // --- encode_mt ---

    #[test]
    fn encode_mt_ctrl_a() {
        assert_eq!(encode_mt(0x01, 0x04), 0x6104);
    }

    #[test]
    fn encode_mt_shift_enter() {
        assert_eq!(encode_mt(0x02, 0x28), 0x6228);
    }

    #[test]
    fn encode_mt_masks_base() {
        // Base keycode should be masked to 8 bits
        assert_eq!(encode_mt(0x01, 0x1FF), encode_mt(0x01, 0xFF));
    }

    // --- filtered_keycodes ---

    #[test]
    fn filter_by_category() {
        let mods = filtered_keycodes(KeycodeCategory::Modifiers, None);
        assert!(mods.len() >= 8); // at least 8 modifiers
        assert!(mods.iter().all(|e| e.category == KeycodeCategory::Modifiers));
    }

    #[test]
    fn filter_by_query() {
        let results = filtered_keycodes(KeycodeCategory::Basic, Some("enter"));
        assert!(results.iter().any(|e| e.code == 0x28));
    }

    #[test]
    fn filter_query_case_insensitive() {
        let results = filtered_keycodes(KeycodeCategory::Basic, Some("SPACE"));
        assert!(results.iter().any(|e| e.code == 0x2C));
    }

    #[test]
    fn filter_no_match() {
        let results = filtered_keycodes(KeycodeCategory::Basic, Some("xyznonexistent"));
        assert!(results.is_empty());
    }

    // --- search_keycodes ---

    #[test]
    fn search_by_name() {
        let results = search_keycodes("Escape");
        assert!(results.iter().any(|e| e.code == 0x29));
    }

    #[test]
    fn search_by_label() {
        let results = search_keycodes("BSPC");
        assert!(results.iter().any(|e| e.code == 0x2A));
    }

    #[test]
    fn search_crosses_categories() {
        let results = search_keycodes("Left");
        // Should find both Left Ctrl (modifier) and Left arrow (navigation)
        let codes: Vec<u16> = results.iter().map(|e| e.code).collect();
        assert!(codes.contains(&0xE0)); // Left Ctrl
        assert!(codes.contains(&0x50)); // Left arrow
    }

    // --- mt_base_keycodes ---

    #[test]
    fn mt_base_excludes_special() {
        let bases = mt_base_keycodes();
        assert!(bases.iter().all(|e| e.category != KeycodeCategory::Special));
        assert!(bases.iter().all(|e| e.code <= 0xFF));
    }

    #[test]
    fn mt_base_includes_letters() {
        let bases = mt_base_keycodes();
        assert!(bases.iter().any(|e| e.code == 0x04)); // A
    }

    // --- category labels ---

    #[test]
    fn all_categories_have_labels() {
        for cat in KeycodeCategory::ALL {
            assert!(!cat.label().is_empty());
        }
    }

    #[test]
    fn all_categories_listed() {
        assert_eq!(KeycodeCategory::ALL.len(), 10);
    }
}
