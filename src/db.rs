use rusqlite::{Connection, Result};
use std::collections::HashMap;
use std::path::Path;
use log::info;

pub fn init(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_hashes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            hash TEXT NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_image_hashes_hash ON image_hashes(hash)",
        (),
    )?;
    Ok(conn)
}

pub fn load_known_hashes(conn: &Connection) -> Result<HashMap<String, Vec<String>>> {
    let mut stmt = conn.prepare("SELECT hash, file_path FROM image_hashes")?;
    let rows = stmt.query_map((), |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for row in rows {
        let (hash, file_path) = row?;
        map.entry(hash).or_insert_with(Vec::new).push(file_path);
    }
    Ok(map)
}

pub fn record_file(conn: &Connection, hash: &str, file_path: &str) -> Result<()> {
    info!("File to record: {}", file_path);
    conn.execute(
        "INSERT OR IGNORE INTO image_hashes (hash, file_path) VALUES (?1, ?2)",
        (hash, file_path),
    )?;
    Ok(())
}
