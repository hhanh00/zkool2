//! Plugin database CRUD — stores plugin metadata in the `plugins` SQLite table.

use anyhow::Result;
use sqlx::{Row, SqliteConnection};

/// Plugin metadata stored in the database.
pub struct PluginRow {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub min_app_version: String,
    pub types: String,       // JSON array: ["memo"]
    pub enabled: bool,
    pub install_dir: String,
    pub script: String,
    pub manifest_json: String,
}

pub async fn list_plugins(conn: &mut SqliteConnection) -> Result<Vec<PluginRow>> {
    let rows = sqlx::query(
        "SELECT id, name, version, author, description, min_app_version, types, enabled, install_dir, script, manifest_json FROM plugins ORDER BY name",
    )
    .map(|row: sqlx::sqlite::SqliteRow| PluginRow {
        id: row.get(0),
        name: row.get(1),
        version: row.get(2),
        author: row.get(3),
        description: row.get(4),
        min_app_version: row.get(5),
        types: row.get(6),
        enabled: row.get::<i64, _>(7) != 0,
        install_dir: row.get(8),
        script: row.get(9),
        manifest_json: row.get(10),
    })
    .fetch_all(&mut *conn)
    .await?;

    Ok(rows)
}

pub async fn get_plugin(conn: &mut SqliteConnection, id: &str) -> Result<Option<PluginRow>> {
    let row = sqlx::query(
        "SELECT id, name, version, author, description, min_app_version, types, enabled, install_dir, script, manifest_json FROM plugins WHERE id = ?",
    )
    .bind(id)
    .map(|row: sqlx::sqlite::SqliteRow| PluginRow {
        id: row.get(0),
        name: row.get(1),
        version: row.get(2),
        author: row.get(3),
        description: row.get(4),
        min_app_version: row.get(5),
        types: row.get(6),
        enabled: row.get::<i64, _>(7) != 0,
        install_dir: row.get(8),
        script: row.get(9),
        manifest_json: row.get(10),
    })
    .fetch_optional(&mut *conn)
    .await?;

    Ok(row)
}

pub async fn upsert_plugin(conn: &mut SqliteConnection, plugin: &PluginRow) -> Result<()> {
    sqlx::query(
        "INSERT OR REPLACE INTO plugins(id, name, version, author, description, min_app_version, types, enabled, install_dir, script, manifest_json) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&plugin.id)
    .bind(&plugin.name)
    .bind(&plugin.version)
    .bind(&plugin.author)
    .bind(&plugin.description)
    .bind(&plugin.min_app_version)
    .bind(&plugin.types)
    .bind(plugin.enabled as i64)
    .bind(&plugin.install_dir)
    .bind(&plugin.script)
    .bind(&plugin.manifest_json)
    .execute(&mut *conn)
    .await?;

    Ok(())
}

pub async fn delete_plugin(conn: &mut SqliteConnection, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM plugins WHERE id = ?")
        .bind(id)
        .execute(&mut *conn)
        .await?;

    Ok(())
}

pub async fn set_plugin_enabled(
    conn: &mut SqliteConnection,
    id: &str,
    enabled: bool,
) -> Result<()> {
    sqlx::query("UPDATE plugins SET enabled = ? WHERE id = ?")
        .bind(enabled as i64)
        .bind(id)
        .execute(&mut *conn)
        .await?;

    Ok(())
}
