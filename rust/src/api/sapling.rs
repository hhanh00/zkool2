use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{anyhow, Context, Result};

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

// Sapling parameter constants — must match those in zcash_proofs.
const SAPLING_SPEND_NAME: &str = "sapling-spend.params";
const SAPLING_OUTPUT_NAME: &str = "sapling-output.params";
const SAPLING_SPEND_HASH: &str =
    "8270785a1a0d0bc77196f000ee6d221c9c9894f55307bd9357c3f0105d31ca63991ab91324160d8f53e2bbd3c2633a6eb8bdf5205d822e7f3f73edac51b2b70c";
const SAPLING_OUTPUT_HASH: &str =
    "657e3d38dbb5cb5e7dd2970e8b03d69b4787dd907285b5a7f0790dcc8072f60bf593b32cc2d1c030e00ff5ae64bf84c5c3beb84ddc841d48264b4a171744d028";
const SAPLING_SPEND_BYTES: u64 = 47_958_396;
const SAPLING_OUTPUT_BYTES: u64 = 3_592_860;
const DOWNLOAD_URL: &str = "https://download.z.cash/downloads";

/// Custom Sapling parameters directory, set on platforms where
/// `zcash_proofs::default_params_folder()` is not available (e.g. Android).
static SAFLING_PARAMS_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Set a custom directory for Sapling parameters.
///
/// Used on Android where the app's documents directory is passed from Dart
/// so that parameters are stored in a writable location.
#[allow(dead_code)]
pub(crate) fn set_sapling_params_dir(dir: PathBuf) {
    let _ = SAFLING_PARAMS_DIR.set(dir);
}

/// Resolve the Sapling parameters directory.
///
/// Returns the custom directory if set (via `set_sapling_params_dir`),
/// otherwise falls back to `zcash_proofs::default_params_folder()`.
pub(crate) fn resolve_params_dir() -> Option<PathBuf> {
    SAFLING_PARAMS_DIR
        .get()
        .cloned()
        .or_else(zcash_proofs::default_params_folder)
}

/// Status of the Sapling proving parameters on disk.
#[cfg_attr(feature = "flutter", frb)]
pub struct SaplingParamsStatus {
    pub downloaded: bool,
}

/// Check whether Sapling parameters are already on disk.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn check_sapling_params() -> SaplingParamsStatus {
    let params_dir = resolve_params_dir();
    let downloaded = params_dir
        .map(|dir| {
            dir.join(SAPLING_SPEND_NAME).exists()
                && dir.join(SAPLING_OUTPUT_NAME).exists()
        })
        .unwrap_or(false);
    SaplingParamsStatus { downloaded }
}

/// Download Sapling parameters from the z.cash download server.
///
/// Verifies file size and Blake2b hash upon download.
/// Safe to call even if they are already downloaded (no-op if valid).
#[cfg_attr(feature = "flutter", frb)]
pub async fn download_sapling_params() -> Result<()> {
    let params_dir = resolve_params_dir()
        .context("Could not resolve Sapling parameters directory")?;

    // Ensure the params directory exists.
    std::fs::create_dir_all(&params_dir)
        .with_context(|| format!("Failed to create params directory: {:?}", params_dir))?;

    download_and_verify(&params_dir, SAPLING_SPEND_NAME, SAPLING_SPEND_HASH, SAPLING_SPEND_BYTES)
        .await
        .context("Failed to download/verify sapling-spend.params")?;

    download_and_verify(&params_dir, SAPLING_OUTPUT_NAME, SAPLING_OUTPUT_HASH, SAPLING_OUTPUT_BYTES)
        .await
        .context("Failed to download/verify sapling-output.params")?;

    Ok(())
}

/// Download a single parameter file from `download.z.cash`, verify its size
/// and Blake2b hash, then save it to the given directory.
///
/// Skips download if an already-validated file exists at the target location.
async fn download_and_verify(
    dir: &PathBuf,
    name: &str,
    expected_hash: &str,
    expected_bytes: u64,
) -> Result<PathBuf> {
    let file_path = dir.join(name);

    // Check if a valid file already exists on disk.
    if let Ok(meta) = std::fs::metadata(&file_path) {
        if meta.len() == expected_bytes {
            match std::fs::read(&file_path) {
                Ok(data) => {
                    let hash = blake2b_simd::blake2b(&data);
                    if hex::encode(hash.as_bytes()) == expected_hash {
                        tracing::info!("Sapling parameter {} already on disk, verified OK", name);
                        return Ok(file_path);
                    }
                }
                Err(e) => tracing::warn!(
                    "Could not read existing param file {:?}: {e} — will re-download",
                    file_path
                ),
            }
        }
        // File exists but is corrupt — remove it and re-download.
        tracing::warn!(
            "Sapling parameter file {:?} is corrupt or wrong size — re-downloading",
            file_path
        );
        let _ = std::fs::remove_file(&file_path);
    }

    // Download the two parts and concatenate them.
    tracing::info!("Downloading Sapling parameter {} …", name);
    let client = reqwest::Client::new();

    let part1_url = format!("{}/{}.part.1", DOWNLOAD_URL, name);
    let part1_resp = client
        .get(&part1_url)
        .send()
        .await
        .with_context(|| format!("Failed to download {part1_url}"))?;
    let part1_bytes = part1_resp
        .bytes()
        .await
        .with_context(|| format!("Failed to read response body from {part1_url}"))?;

    let mut combined: Vec<u8> = part1_bytes.to_vec();

    // The second part may be needed for files larger than the CloudFlare cache limit.
    if (combined.len() as u64) < expected_bytes {
        let part2_url = format!("{}/{}.part.2", DOWNLOAD_URL, name);
        tracing::info!(
            "Part 1 only had {} bytes — downloading {}",
            combined.len(),
            part2_url
        );
        let part2_resp = client
            .get(&part2_url)
            .send()
            .await
            .with_context(|| format!("Failed to download {part2_url}"))?;
        let part2_bytes = part2_resp
            .bytes()
            .await
            .with_context(|| format!("Failed to read response body from {part2_url}"))?;
        combined.extend_from_slice(&part2_bytes);
    }

    // Truncate to the expected size (the chain produces exactly expected_bytes).
    combined.truncate(expected_bytes as usize);

    // Verify size.
    if (combined.len() as u64) != expected_bytes {
        return Err(anyhow!(
            "{name} size mismatch: expected {expected_bytes} bytes, got {} bytes",
            combined.len()
        ));
    }

    // Verify Blake2b hash.
    let hash = blake2b_simd::blake2b(&combined);
    let hash_hex = hex::encode(hash.as_bytes());
    if hash_hex != expected_hash {
        return Err(anyhow!(
            "{name} hash mismatch:\n  expected: {expected_hash}\n  got:      {hash_hex}"
        ));
    }

    // Write the verified parameters to disk.
    std::fs::write(&file_path, &combined)
        .with_context(|| format!("Failed to write parameter file {:?}", file_path))?;

    tracing::info!("Successfully downloaded and verified {name}");
    Ok(file_path)
}
