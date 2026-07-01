use anyhow::Result;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

/// Status of the Sapling proving parameters on disk.
#[cfg_attr(feature = "flutter", frb)]
pub struct SaplingParamsStatus {
    pub downloaded: bool,
}

/// Check whether Sapling parameters are already on disk.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn check_sapling_params() -> SaplingParamsStatus {
    let params_dir = zcash_proofs::default_params_folder();
    let downloaded = params_dir
        .map(|dir| {
            dir.join(zcash_proofs::SAPLING_SPEND_NAME).exists()
                && dir.join(zcash_proofs::SAPLING_OUTPUT_NAME).exists()
        })
        .unwrap_or(false);
    SaplingParamsStatus { downloaded }
}

/// Download Sapling parameters from the z.cash download server.
/// Verifies file size and hash upon download.
/// Safe to call even if they are already downloaded (no-op if valid).
#[cfg_attr(feature = "flutter", frb)]
pub async fn download_sapling_params() -> Result<()> {
    zcash_proofs::download_sapling_parameters(None)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("Failed to download Sapling parameters: {e}"))
}
