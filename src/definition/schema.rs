use serde::Deserialize;

/// VIA keyboard definition from usevia.app.
///
/// Both v2 and v3 hosted definitions use the same format:
/// - `vendorProductId`: combined u32
/// - `layouts.keys`: array of positioned key objects (NOT KLE format)
/// - `lighting`: can be a string ("none") or an object with extends/effects
#[derive(Debug, Clone, Deserialize)]
pub struct ViaDefinition {
    pub name: String,
    #[serde(rename = "vendorProductId")]
    pub vendor_product_id: u32,
    #[serde(default)]
    pub lighting: serde_json::Value,
    pub matrix: MatrixInfo,
    pub layouts: Layouts,
    #[serde(default)]
    pub menus: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MatrixInfo {
    pub rows: u8,
    pub cols: u8,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Layouts {
    #[serde(default)]
    pub width: Option<f64>,
    #[serde(default)]
    pub height: Option<f64>,
    /// Key position/size data — this is the usevia.app format (NOT KLE).
    #[serde(default)]
    pub keys: Vec<KeyDefinition>,
    /// KLE-format keymap array — used by some local/community definitions.
    #[serde(default)]
    pub keymap: Vec<serde_json::Value>,
    #[serde(default)]
    pub labels: Option<Vec<serde_json::Value>>,
    /// Layout options keyed by group index, then option index.
    /// e.g. `{ "0": { "0": [keys...], "1": [keys...] } }`
    /// Option "0" is the default for each group.
    #[serde(default, rename = "optionKeys")]
    pub option_keys: std::collections::HashMap<String, std::collections::HashMap<String, Vec<KeyDefinition>>>,
}

/// A key in the usevia.app layout format.
#[derive(Debug, Clone, Deserialize)]
pub struct KeyDefinition {
    /// Matrix row
    pub row: u8,
    /// Matrix column
    pub col: u8,
    /// X position (in key units, e.g. 0, 1, 1.25)
    pub x: f64,
    /// Y position
    pub y: f64,
    /// Width (default 1.0)
    #[serde(default = "default_one")]
    pub w: f64,
    /// Height (default 1.0)
    #[serde(default = "default_one")]
    pub h: f64,
    /// Rotation angle in degrees
    #[serde(default)]
    pub r: f64,
    /// Rotation origin X
    #[serde(default)]
    pub rx: f64,
    /// Rotation origin Y
    #[serde(default)]
    pub ry: f64,
    /// Whether this key is a "decal" (decorative, not functional)
    #[serde(default)]
    pub d: bool,
    /// Key color hint
    #[serde(default)]
    pub color: Option<String>,
    /// Encoder index (if this key represents a rotary encoder)
    #[serde(default)]
    pub ei: Option<u8>,
}

fn default_one() -> f64 {
    1.0
}

impl ViaDefinition {
    pub fn vendor_id(&self) -> u16 {
        (self.vendor_product_id >> 16) as u16
    }

    pub fn product_id(&self) -> u16 {
        (self.vendor_product_id & 0xFFFF) as u16
    }
}
