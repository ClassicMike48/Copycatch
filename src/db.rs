use log::info;
use rusqlite::{Connection, Result};
use std::path::Path;

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

pub fn record_file(conn: &Connection, hash: &str, file_path: &str) -> Result<()> {
    info!("File to record: {}", file_path);
    conn.execute(
        "INSERT OR IGNORE INTO image_hashes (hash, file_path) VALUES (?1, ?2)",
        (hash, file_path),
    )?;
    Ok(())
}

pub fn hash_exists(conn: &Connection, hash: &str) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM image_hashes WHERE hash = ?1)",
        [hash],
        |row| row.get(0),
    )
}
