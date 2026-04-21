use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_definition() {
        let json = r#"{
            "name": "Test Keyboard",
            "vendorProductId": 305419896,
            "matrix": { "rows": 4, "cols": 12 },
            "layouts": { "keys": [] }
        }"#;
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.name, "Test Keyboard");
        assert_eq!(def.matrix.rows, 4);
        assert_eq!(def.matrix.cols, 12);
        assert!(def.layouts.keys.is_empty());
    }

    #[test]
    fn vendor_product_id_split() {
        let json = r#"{
            "name": "Test",
            "vendorProductId": 305419896,
            "matrix": { "rows": 1, "cols": 1 },
            "layouts": { "keys": [] }
        }"#;
        // 305419896 = 0x12345678
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.vendor_id(), 0x1234);
        assert_eq!(def.product_id(), 0x5678);
    }

    #[test]
    fn parse_with_lighting_none() {
        let json = r#"{
            "name": "Test",
            "vendorProductId": 0,
            "lighting": "none",
            "matrix": { "rows": 1, "cols": 1 },
            "layouts": { "keys": [] }
        }"#;
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.lighting, serde_json::Value::String("none".to_string()));
    }

    #[test]
    fn parse_with_lighting_object() {
        let json = r#"{
            "name": "Test",
            "vendorProductId": 0,
            "lighting": { "extends": "qmk_rgblight", "effects": [] },
            "matrix": { "rows": 1, "cols": 1 },
            "layouts": { "keys": [] }
        }"#;
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert!(def.lighting.is_object());
    }

    #[test]
    fn lighting_defaults_to_null() {
        let json = r#"{
            "name": "Test",
            "vendorProductId": 0,
            "matrix": { "rows": 1, "cols": 1 },
            "layouts": { "keys": [] }
        }"#;
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert!(def.lighting.is_null());
    }

    #[test]
    fn parse_key_definition() {
        let json = r#"{
            "name": "Test",
            "vendorProductId": 0,
            "matrix": { "rows": 2, "cols": 3 },
            "layouts": {
                "keys": [
                    { "row": 0, "col": 0, "x": 0, "y": 0 },
                    { "row": 0, "col": 1, "x": 1, "y": 0, "w": 1.5, "h": 2.0 },
                    { "row": 1, "col": 0, "x": 0, "y": 1, "d": true }
                ]
            }
        }"#;
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.layouts.keys.len(), 3);

        let k0 = &def.layouts.keys[0];
        assert_eq!(k0.row, 0);
        assert_eq!(k0.col, 0);
        assert_eq!(k0.w, 1.0); // default
        assert_eq!(k0.h, 1.0); // default
        assert!(!k0.d);

        let k1 = &def.layouts.keys[1];
        assert_eq!(k1.w, 1.5);
        assert_eq!(k1.h, 2.0);

        let k2 = &def.layouts.keys[2];
        assert!(k2.d); // decal
    }

    #[test]
    fn parse_with_option_keys() {
        let json = r#"{
            "name": "Test",
            "vendorProductId": 0,
            "matrix": { "rows": 1, "cols": 2 },
            "layouts": {
                "keys": [{ "row": 0, "col": 0, "x": 0, "y": 0 }],
                "optionKeys": {
                    "0": {
                        "0": [{ "row": 0, "col": 1, "x": 1, "y": 0 }],
                        "1": [{ "row": 0, "col": 1, "x": 1, "y": 0, "w": 2.0 }]
                    }
                }
            }
        }"#;
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert_eq!(def.layouts.keys.len(), 1);
        assert!(def.layouts.option_keys.contains_key("0"));
        assert!(def.layouts.option_keys["0"].contains_key("0"));
        assert!(def.layouts.option_keys["0"].contains_key("1"));
    }

    #[test]
    fn parse_menus_default_empty() {
        let json = r#"{
            "name": "Test",
            "vendorProductId": 0,
            "matrix": { "rows": 1, "cols": 1 },
            "layouts": { "keys": [] }
        }"#;
        let def: ViaDefinition = serde_json::from_str(json).unwrap();
        assert!(def.menus.is_empty());
    }

    #[test]
    fn matrix_info_serde() {
        let m = MatrixInfo { rows: 5, cols: 14 };
        let json = serde_json::to_string(&m).unwrap();
        let restored: MatrixInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.rows, 5);
        assert_eq!(restored.cols, 14);
    }
}
