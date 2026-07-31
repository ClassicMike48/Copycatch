use log::info;
use rusqlite::{Connection, Result};
use std::path::Path;
use crate::file_walk::{HexString, PHashHexString};
pub fn init(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute("PRAGMA foreign_keys = ON", ())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_hashes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            crypto_hash TEXT NOT NULL,
            p_hash TEXT NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_image_hashes_hash ON image_hashes(crypto_hash)",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS phash_comparisons (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            image_id_1 INTEGER NOT NULL REFERENCES image_hashes(id),
            image_id_2 INTEGER NOT NULL REFERENCES image_hashes(id),
            hamming_distance INTEGER NOT NULL,
            compared_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            CHECK (image_id_1 < image_id_2),
            UNIQUE (image_id_1, image_id_2)
        )",
        (),
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_phash_comparisons_image_id_2 ON phash_comparisons(image_id_2)",
        (),
    )?;

    Ok(conn)
}

pub fn record_file(
    conn: &Connection,
    crypto_hash: &HexString,
    p_hash: &PHashHexString,
    file_path: &str,
) -> Result<()> {
    info!("File to record: {}", file_path);
    conn.execute(
        "INSERT OR IGNORE INTO image_hashes (crypto_hash, p_hash, file_path) VALUES (?1, ?2, ?3)",
        (crypto_hash.get_hex(), p_hash.get_hex(), file_path),
    )?;
    Ok(())
}

pub fn crypto_hash_exists(conn: &Connection, crypto_hash: &HexString) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM image_hashes WHERE crypto_hash = ?1)",
        [crypto_hash.get_hex()],
        |row| row.get(0),
    )
}

#[derive(Debug, Clone)]
pub struct ImageRecord {
    id: i64,
    crypto_hash: HexString,
    p_hash: PHashHexString,
    file_path: String,
    created_at: String,
}

pub fn get_images(conn: &Connection) -> Result<Vec<ImageRecord>> {
    let mut  stmt = conn.prepare(
        "SELECT id, crypto_hash, p_hash, file_path, created_at FROM image_hashes",
    )?;
    let rows = stmt.query_map((), |row| {
        Ok(ImageRecord {
            id: row.get(0)?,
            crypto_hash: HexString::new(row.get(1)?),
            p_hash: PHashHexString::new(row.get(2)?),
            file_path: row.get(3)?,
            created_at: row.get(4)?,
        })
    })?;
    rows.collect()
}