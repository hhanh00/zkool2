//! Plugin system core: types, engine factory, dispatch, install/remove.
//!
//! ## Plugin lifecycle
//! 1. **Install**: download archive → extract to `$DATADIR/plugins/<id>/` → validate manifest →
//!    create engine → compile script → call `get_prefixes()` → store in DB.
//! 2. **Load (startup)**: read all enabled plugins from DB → compile AST → cache AST.
//! 3. **Dispatch**: given memo bytes, find plugins whose prefixes match → call `process_memo()`.
//! 4. **Remove**: delete plugin directory → remove DB row → evict AST cache.

pub mod db;
pub mod rhai_api;

use anyhow::{anyhow, bail, Context, Result};
use rhai::{AST, Dynamic, Engine, Scope};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::OnceLock;

use crate::api::coin::Coin;
use crate::plugin::db as plugin_db;
use crate::plugin::rhai_api::{
    create_sandboxed_engine, extract_prefixes, extract_sections, with_memo_bytes,
    ParsedMemoCell, ParsedMemoSection,
};

// ── AST cache ───────────────────────────────────────────────────────────

static AST_CACHE: OnceLock<Mutex<HashMap<String, AST>>> = OnceLock::new();

fn ast_cache() -> &'static Mutex<HashMap<String, AST>> {
    AST_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cache_ast(plugin_id: &str, ast: AST) {
    if let Ok(mut cache) = ast_cache().lock() {
        cache.insert(plugin_id.to_string(), ast);
    }
}

fn get_cached_ast(plugin_id: &str) -> Option<AST> {
    ast_cache().lock().ok()?.get(plugin_id).cloned()
}

fn evict_ast(plugin_id: &str) {
    if let Ok(mut cache) = ast_cache().lock() {
        cache.remove(plugin_id);
    }
}

// ── Plugin index cache ──────────────────────────────────────────────────

/// Cached metadata for a loaded memo plugin.
#[derive(Clone)]
struct CachedMemoPlugin {
    id: String,
    script: String,
    /// 4-byte prefixes this plugin handles.
    prefixes: Vec<Vec<u8>>,
}

/// In-memory index of enabled memo plugins and their prefixes.
/// Avoids DB queries and rhai function calls on every memo.
static PLUGIN_INDEX: LazyLock<Mutex<Vec<CachedMemoPlugin>>> =
    LazyLock::new(|| Mutex::new(Vec::new()));

/// Rebuild the plugin index from the database.
pub async fn refresh_plugin_index(c: &Coin) -> Result<()> {
    let mut conn = c.get_connection().await?;
    let rows = plugin_db::list_plugins(&mut conn).await?;
    let engine = create_sandboxed_engine();
    let mut index = Vec::new();
    for row in rows {
        if !row.enabled {
            continue;
        }
        let types: Vec<String> = serde_json::from_str(&row.types).unwrap_or_default();
        if !types.contains(&"memo".to_string()) {
            continue;
        }
        let ast = match compile_plugin(&row.id, &row.script) {
            Ok(ast) => ast,
            Err(e) => {
                tracing::warn!("Failed to compile plugin '{}': {e}", row.id);
                continue;
            }
        };
        let hex_prefixes = discover_prefixes(&engine, &ast).unwrap_or_default();
        let prefixes: Vec<Vec<u8>> = hex_prefixes
            .iter()
            .filter_map(|h| hex::decode(h).ok())
            .filter(|b| b.len() == 4)
            .collect();
        index.push(CachedMemoPlugin {
            id: row.id.clone(),
            script: row.script,
            prefixes,
        });
    }
    if let Ok(mut guard) = PLUGIN_INDEX.lock() {
        *guard = index;
    }
    Ok(())
}

// ── Section output types ────────────────────────────────────────────────

/// A memo section returned by a plugin — a titled table with typed cells.
#[derive(Debug, Clone)]
pub struct MemoSection {
    pub title: String,
    pub headers: Vec<String>,
    pub rows: Vec<MemoRow>,
}

#[derive(Debug, Clone)]
pub struct MemoRow {
    pub cells: Vec<MemoCell>,
}

#[derive(Debug, Clone)]
pub struct MemoCell {
    pub cell_type: String, // "number" | "string" | "date"
    pub value: String,
}

// ── Manifest ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    pub min_app_version: String,
    pub types: Vec<String>,
    #[serde(default = "default_entry_point")]
    pub entry_point: String,
}

fn default_entry_point() -> String {
    "main.rhai".to_string()
}

// ── Runtime plugin ──────────────────────────────────────────────────────

pub struct Plugin {
    pub manifest: PluginManifest,
    pub enabled: bool,
    pub script: String,
    pub install_dir: String,
    /// Cached prefixes discovered by calling get_prefixes().
    /// Keyed by type name, e.g. "memo" → vec of 4-byte prefixes.
    pub prefixes: HashMap<String, Vec<Vec<u8>>>,
    /// Compiled AST for the plugin script (may be cached globally).
    pub ast: Option<AST>,
}

// ── Plugin load ─────────────────────────────────────────────────────────

/// Call `get_prefixes()` on a plugin script and cache the result.
pub fn discover_prefixes(engine: &Engine, ast: &AST) -> Result<Vec<String>> {
    let result: Dynamic = engine
        .call_fn(&mut Scope::new(), ast, "get_prefixes", ())
        .map_err(|e| anyhow!("get_prefixes() failed: {e}"))?;
    Ok(extract_prefixes(result))
}

/// Compile a plugin script into an AST and cache it.
pub fn compile_plugin(plugin_id: &str, script: &str) -> Result<AST> {
    if let Some(ast) = get_cached_ast(plugin_id) {
        return Ok(ast);
    }
    let engine = create_sandboxed_engine();
    let ast = engine
        .compile(script)
        .map_err(|e| anyhow!("Failed to compile plugin '{plugin_id}': {e}"))?;
    cache_ast(plugin_id, ast.clone());
    Ok(ast)
}

// ── Memo dispatch ───────────────────────────────────────────────────────

/// Parse a memo with all matching plugins.
/// Uses an in-memory plugin index to avoid DB queries per memo.
pub async fn parse_memo_with_plugins(c: &Coin, memo_bytes: &[u8]) -> Result<Vec<MemoSection>> {
    if memo_bytes.is_empty() || memo_bytes[0] != 0xFF {
        return Ok(vec![]);
    }
    let payload = &memo_bytes[1..];
    if payload.len() < 4 {
        return Ok(vec![]);
    }
    let prefix = &payload[0..4];

    // Clone out of lock — must not hold MutexGuard across await
    let plugins: Vec<CachedMemoPlugin> = {
        PLUGIN_INDEX.lock().unwrap().clone()
    };
    let plugins = if plugins.is_empty() {
        refresh_plugin_index(c).await?;
        PLUGIN_INDEX.lock().unwrap().clone()
    } else {
        plugins
    };

    let mut sections = Vec::new();
    let engine = create_sandboxed_engine();

    for plugin in &plugins {
        if !plugin.prefixes.iter().any(|p| p.as_slice() == prefix) {
            continue;
        }

        let ast = match compile_plugin(&plugin.id, &plugin.script) {
            Ok(ast) => ast,
            Err(e) => {
                tracing::warn!("Failed to compile plugin: {e}");
                continue;
            }
        };

        with_memo_bytes(payload, || {
            match engine.call_fn::<Dynamic>(&mut Scope::new(), &ast, "process_memo", ()) {
                Ok(result) => {
                    for section in extract_sections(result) {
                        sections.push(MemoSection::from(section));
                    }
                }
                Err(e) => {
                    tracing::warn!("Plugin process_memo failed: {e}");
                }
            }
        });
    }

    Ok(sections)
}

// ── Install / remove ────────────────────────────────────────────────────

/// Download and install a plugin from a URL.
pub async fn install_plugin_from_url(c: &Coin, url: &str) -> Result<Plugin> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await
        .context("Failed to download plugin")?;
    let archive_bytes = response
        .bytes()
        .await
        .context("Failed to read plugin archive")?;

    install_plugin_from_bytes(c, &archive_bytes).await
}

/// Install a plugin from an archive (zip). Works entirely in memory — no disk writes.
pub async fn install_plugin_from_bytes(c: &Coin, archive: &[u8]) -> Result<Plugin> {
    let cursor = Cursor::new(archive);
    let mut zip =
        zip::ZipArchive::new(cursor).context("Failed to open plugin archive")?;

    // Read manifest.json from zip
    let manifest_entry = zip
        .by_name("manifest.json")
        .context("Archive missing manifest.json — zip must be flat (no subdirectories)")?;
    let manifest: PluginManifest = serde_json::from_reader(manifest_entry)
        .context("Failed to parse manifest.json from archive")?;

    // Validate
    if manifest.id.is_empty()
        || manifest.id.contains("..")
        || manifest.id.contains('/')
        || manifest.id.contains('\\')
    {
        bail!("Invalid plugin id: {}", manifest.id);
    }
    if manifest.types.is_empty() {
        bail!("Plugin must support at least one type");
    }

    // Read entry point script from zip
    let script_entry = zip
        .by_name(&manifest.entry_point)
        .context(format!("Archive missing {}", manifest.entry_point))?;
    let script = std::io::read_to_string(script_entry)
        .context("Failed to read script from archive")?;

    // Compile and discover prefixes
    let ast = compile_plugin(&manifest.id, &script)?;
    let engine = create_sandboxed_engine();
    let mut prefixes: HashMap<String, Vec<Vec<u8>>> = HashMap::new();
    if manifest.types.contains(&"memo".to_string()) {
        let hex_prefixes = discover_prefixes(&engine, &ast).unwrap_or_default();
        let byte_prefixes: Vec<Vec<u8>> = hex_prefixes
            .iter()
            .filter_map(|h| hex::decode(h).ok())
            .filter(|b| b.len() == 4)
            .collect();
        if !byte_prefixes.is_empty() {
            prefixes.insert("memo".to_string(), byte_prefixes);
        }
    }

    let plugin = Plugin {
        manifest: manifest.clone(),
        enabled: true,
        script: script.clone(),
        install_dir: String::new(), // no disk extraction needed
        prefixes,
        ast: Some(ast),
    };

    // Store in DB
    let types_json = serde_json::to_string(&plugin.manifest.types)?;
    let manifest_json = serde_json::to_string(&plugin.manifest)?;
    let mut conn = c.get_connection().await?;
    plugin_db::upsert_plugin(
        &mut conn,
        &plugin_db::PluginRow {
            id: plugin.manifest.id.clone(),
            name: plugin.manifest.name.clone(),
            version: plugin.manifest.version.clone(),
            author: plugin.manifest.author.clone(),
            description: plugin.manifest.description.clone(),
            min_app_version: plugin.manifest.min_app_version.clone(),
            types: types_json,
            enabled: plugin.enabled,
            install_dir: plugin.install_dir.clone(),
            script: plugin.script.clone(),
            manifest_json,
        },
    )
    .await?;

    refresh_plugin_index(c).await?;
    Ok(plugin)
}

/// Remove a plugin: delete DB row, evict AST cache, refresh index.
pub async fn remove_plugin(c: &Coin, id: &str) -> Result<()> {
    let mut conn = c.get_connection().await?;
    let row = plugin_db::get_plugin(&mut conn, id).await?;
    plugin_db::delete_plugin(&mut conn, id).await?;

    if let Some(row) = row {
        if !row.install_dir.is_empty() {
            let dir = std::path::Path::new(&row.install_dir);
            if dir.exists() {
                let _ = std::fs::remove_dir_all(dir);
            }
        }
    }

    evict_ast(id);
    refresh_plugin_index(c).await?;
    Ok(())
}

/// Enable or disable a plugin.
pub async fn set_plugin_enabled(c: &Coin, id: &str, enabled: bool) -> Result<()> {
    let mut conn = c.get_connection().await?;
    plugin_db::set_plugin_enabled(&mut conn, id, enabled).await?;

    if !enabled {
        evict_ast(id);
    }
    refresh_plugin_index(c).await?;

    Ok(())
}

/// List all installed plugins.
pub async fn list_plugins(c: &Coin) -> Result<Vec<plugin_db::PluginRow>> {
    let mut conn = c.get_connection().await?;
    plugin_db::list_plugins(&mut conn).await
}

/// Initialize the plugin system at startup (no-op: plugins live in DB).
pub fn init_plugins_dir() -> Result<()> {
    Ok(())
}

// ── Conversions ─────────────────────────────────────────────────────────

impl From<ParsedMemoSection> for MemoSection {
    fn from(s: ParsedMemoSection) -> Self {
        MemoSection {
            title: s.title,
            headers: s.headers,
            rows: s
                .rows
                .into_iter()
                .map(|cells| MemoRow {
                    cells: cells.into_iter().map(MemoCell::from).collect(),
                })
                .collect(),
        }
    }
}

impl From<ParsedMemoCell> for MemoCell {
    fn from(c: ParsedMemoCell) -> Self {
        MemoCell {
            cell_type: c.cell_type,
            value: c.value,
        }
    }
}
