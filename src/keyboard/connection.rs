use anyhow::{Result, bail};

use qmk_via_api::api::KeyboardApi;
use qmk_via_api::scan::{KeyboardDeviceInfo, scan_keyboards};

use crate::definition::schema::MatrixInfo;

/// Wraps a connected keyboard with its protocol info.
pub struct KeyboardConnection {
    pub api: KeyboardApi,
    pub protocol_version: u16,
    pub layer_count: u8,
    pub device_info: KeyboardDeviceInfo,
}

/// Convert qmk_via_api::Error to anyhow::Error since it doesn't impl std::error::Error.
fn via_err(msg: &str) -> impl FnOnce(qmk_via_api::Error) -> anyhow::Error + '_ {
    move |e| anyhow::anyhow!("{msg}: {e:?}")
}

/// Scan for VIA-compatible keyboards.
pub fn scan_devices() -> Result<Vec<KeyboardDeviceInfo>> {
    scan_keyboards().map_err(via_err("Failed to scan for keyboards"))
}

impl KeyboardConnection {
    /// Connect to a keyboard device and read its protocol info.
    pub fn connect(device: &KeyboardDeviceInfo) -> Result<Self> {
        let api = KeyboardApi::from_device(device)
            .map_err(via_err("Failed to open keyboard HID device"))?;

        let protocol_version = api
            .get_protocol_version()
            .map_err(via_err("Failed to get protocol version"))?;

        if protocol_version == 0 || protocol_version == 0xFFFF {
            bail!(
                "Invalid protocol version {:#06x} — device may not support VIA",
                protocol_version
            );
        }

        let layer_count = api
            .get_layer_count()
            .map_err(via_err("Failed to get layer count"))?;

        Ok(Self {
            api,
            protocol_version,
            layer_count,
            device_info: device.clone(),
        })
    }

    /// Read the full keymap for all layers.
    /// Returns layers[layer_idx] = Vec<u16> of keycodes indexed by (row * cols + col).
    pub fn read_all_layers(&self, matrix: &MatrixInfo) -> Result<Vec<Vec<u16>>> {
        let mut layers = Vec::new();

        let matrix_info = qmk_via_api::api::MatrixInfo {
            rows: matrix.rows,
            cols: matrix.cols,
        };

        for layer in 0..self.layer_count {
            let keycodes = self
                .api
                .read_raw_matrix(matrix_info, layer)
                .map_err(via_err(&format!("Failed to read layer {layer}")))?;
            layers.push(keycodes);
        }

        Ok(layers)
    }

    /// Write a single keycode to the device.
    pub fn set_keycode(&self, layer: u8, row: u8, col: u8, keycode: u16) -> Result<()> {
        self.api
            .set_key(layer, row, col, keycode)
            .map_err(via_err(&format!(
                "Failed to set keycode at layer={layer} row={row} col={col}"
            )))?;
        Ok(())
    }
}
