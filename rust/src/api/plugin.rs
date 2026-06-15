use anyhow::Result;
use crate::api::coin::Coin;
use crate::plugin;

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

// ── FFI Data Structures ─────────────────────────────────────────────────

/// Plugin metadata visible to the UI.
#[cfg_attr(feature = "flutter", frb)]
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
    pub types: Vec<String>,
    /// Hex-encoded 4-byte prefixes (discovered by calling get_prefixes() at load time)
    pub memo_prefixes: Vec<String>,
}

/// A parsed memo section — a titled table.
#[cfg_attr(feature = "flutter", frb)]
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct MemoSection {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<MemoRow>,
}

/// A row of cells in a memo section.
#[cfg_attr(feature = "flutter", frb)]
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct MemoRow {
    pub cells: Vec<MemoCell>,
}

/// A single typed cell in a memo table.
#[cfg_attr(feature = "flutter", frb)]
#[cfg_attr(feature = "flutter", frb(dart_metadata = ("freezed")))]
pub struct MemoCell {
    /// "number" | "string" | "date"
    pub cell_type: String,
    pub value: String,
}

// ── FFI Functions ───────────────────────────────────────────────────────

/// List all installed plugins.
#[cfg_attr(feature = "flutter", frb)]
pub async fn list_plugins(c: &Coin) -> Result<Vec<PluginInfo>> {
    let mut conn = c.get_connection().await?;
    let rows = plugin::db::list_plugins(&mut conn).await?;
    Ok(rows
        .into_iter()
        .filter_map(|row| {
            let types: Vec<String> = serde_json::from_str(&row.types).ok()?;
            // Re-derive prefixes from the script at call time
            let memo_prefixes = plugin::compile_plugin(&row.id, &row.script)
                .ok()
                .and_then(|ast| {
                    let engine = plugin::rhai_api::create_sandboxed_engine();
                    plugin::discover_prefixes(&engine, &ast).ok()
                })
                .unwrap_or_default();
            Some(PluginInfo {
                id: row.id,
                name: row.name,
                version: row.version,
                author: row.author,
                description: row.description,
                enabled: row.enabled,
                types,
                memo_prefixes,
            })
        })
        .collect())
}

/// Install a plugin from a URL (downloads a .zip archive).
#[cfg_attr(feature = "flutter", frb)]
pub async fn install_plugin(url: String, c: &Coin) -> Result<PluginInfo> {
    let plugin = plugin::install_plugin_from_url(c, &url).await?;
    Ok(PluginInfo {
        id: plugin.manifest.id,
        name: plugin.manifest.name,
        version: plugin.manifest.version,
        author: plugin.manifest.author,
        description: plugin.manifest.description,
        enabled: plugin.enabled,
        types: plugin.manifest.types,
        memo_prefixes: plugin
            .prefixes
            .get("memo")
            .map(|p| p.iter().map(|b| hex::encode(b)).collect())
            .unwrap_or_default(),
    })
}

/// Remove a plugin completely (files + DB).
#[cfg_attr(feature = "flutter", frb)]
pub async fn remove_plugin(id: String, c: &Coin) -> Result<()> {
    plugin::remove_plugin(c, &id).await
}

/// Enable or disable a plugin.
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn set_plugin_enabled(id: String, enabled: bool, c: &Coin) -> Result<()> {
    tokio::runtime::Handle::current().block_on(async {
        plugin::set_plugin_enabled(c, &id, enabled).await
    })
}

/// Parse a memo with all matching plugins.
/// `memo_bytes` is the full 512-byte memo (including the 0xFF type byte).
/// Returns sections from all plugins whose prefixes match.
#[cfg_attr(feature = "flutter", frb)]
pub async fn parse_memo_with_plugins(memo_bytes: Vec<u8>, c: &Coin) -> Result<Vec<MemoSection>> {
    let sections = plugin::parse_memo_with_plugins(c, &memo_bytes).await?;
    Ok(sections
        .into_iter()
        .map(|s| MemoSection {
            title: s.title,
            headers: s.headers,
            rows: s
                .rows
                .into_iter()
                .map(|r| MemoRow {
                    cells: r
                        .cells
                        .into_iter()
                        .map(|c| MemoCell {
                            cell_type: c.cell_type,
                            value: c.value,
                        })
                        .collect(),
                })
                .collect(),
        })
        .collect())
}

/// Initialize the plugin system at app startup (creates plugins directory).
#[cfg_attr(feature = "flutter", frb(sync))]
pub fn init_plugins() -> Result<()> {
    plugin::init_plugins_dir()
}
