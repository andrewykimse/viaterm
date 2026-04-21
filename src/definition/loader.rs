use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::ProjectDirs;

use super::schema::ViaDefinition;

/// Load a VIA keyboard definition from a local JSON file.
pub fn load_definition_file(path: &Path) -> Result<ViaDefinition> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read definition file: {}", path.display()))?;

    let def: ViaDefinition = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse definition file: {}", path.display()))?;

    Ok(def)
}

/// Fetch a keyboard definition by VID/PID from usevia.app, with local caching.
///
/// Tries the local cache first, then falls back to fetching from the network.
/// Protocol version determines whether to use v2 or v3 definitions.
pub fn fetch_definition(vendor_id: u16, product_id: u16, protocol_version: u16) -> Result<ViaDefinition> {
    let vendor_product_id = ((vendor_id as u32) << 16) | (product_id as u32);
    let version = if protocol_version >= 11 { "v3" } else { "v2" };

    // Try cache first
    if let Some(cached) = load_cached(vendor_product_id, version)? {
        tracing::info!(
            "Loaded definition from cache for VID:{:04X} PID:{:04X}",
            vendor_id,
            product_id
        );
        return Ok(cached);
    }

    // Fetch from usevia.app
    let url = format!(
        "https://usevia.app/definitions/{version}/{vendor_product_id}.json"
    );

    tracing::info!("Fetching definition from {}", url);

    let body: String = ureq::get(&url)
        .call()
        .with_context(|| {
            format!(
                "Failed to fetch definition for VID:{vendor_id:04X} PID:{product_id:04X} from {url}"
            )
        })?
        .body_mut()
        .read_to_string()
        .with_context(|| "Failed to read response body")?;

    let def: ViaDefinition = serde_json::from_str(&body).with_context(|| {
        format!(
            "Failed to parse definition JSON from {} (first 200 chars: {})",
            url,
            &body[..body.len().min(200)]
        )
    })?;

    // Cache for next time
    if let Err(e) = save_to_cache(vendor_product_id, version, &body) {
        tracing::warn!("Failed to cache definition: {e}");
    }

    Ok(def)
}

fn cache_dir() -> Option<PathBuf> {
    ProjectDirs::from("", "", "viaterm").map(|dirs| dirs.cache_dir().to_path_buf())
}

fn cache_path(vendor_product_id: u32, version: &str) -> Option<PathBuf> {
    cache_dir().map(|dir| dir.join("definitions").join(version).join(format!("{vendor_product_id}.json")))
}

fn load_cached(vendor_product_id: u32, version: &str) -> Result<Option<ViaDefinition>> {
    let path = match cache_path(vendor_product_id, version) {
        Some(p) if p.exists() => p,
        _ => return Ok(None),
    };

    let content = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read cached definition: {}", path.display()))?;

    let def: ViaDefinition = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse cached definition: {}", path.display()))?;

    Ok(Some(def))
}

fn save_to_cache(vendor_product_id: u32, version: &str, json: &str) -> Result<()> {
    let path = cache_path(vendor_product_id, version)
        .context("Could not determine cache directory")?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create cache dir: {}", parent.display()))?;
    }

    fs::write(&path, json)
        .with_context(|| format!("Failed to write cache file: {}", path.display()))?;

    Ok(())
}
