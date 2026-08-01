use crate::file_walk::{HexString, PHashHexString};
use log::info;
use rusqlite::{Connection, Result};
use std::path::Path;

/// Initializes the SQLite database and creates the necessary tables if they do not exist
///
/// Returns a Connection to the database
pub fn init(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;
    conn.execute("PRAGMA foreign_keys = ON", ())?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS image_hashes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            crypto_hash TEXT NOT NULL UNIQUE,
            p_hash TEXT NOT NULL,
            file_path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
        (),
    )?;

    // The total rows should equal N(N-1)/2 where N is the number of images in the image_hashes table
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

/// Records a file's crypto hash, p_hash, and file path into the image_hashes table
///
/// If the crypto hash already exists, the record will not be inserted again
///
/// Failure
/// Will return Err if sql cannot be converted to a C-compatible string
/// or if the underlying SQLite call fails.
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

/// Checks if a crypto hash exists in the image_hashes table
pub fn crypto_hash_exists(conn: &Connection, crypto_hash: &HexString) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM image_hashes WHERE crypto_hash = ?1)",
        [crypto_hash.get_hex()],
        |row| row.get(0),
    )
}

#[derive(Debug, Clone)]
/// Represents a record in the image_hashes table
pub struct ImageRecord {
    pub crypto_hash: HexString,
    pub p_hash: PHashHexString,
    pub id: i64,
    pub file_path: String,
    pub created_at: String,
}

/// Helper function to turn a single row from rusqlite into an ImageRecord struct
fn image_record_from_row(row: &rusqlite::Row) -> Result<ImageRecord> {
    Ok(ImageRecord {
        id: row.get(0)?,
        crypto_hash: HexString::new(row.get(1)?),
        p_hash: PHashHexString::new(row.get(2)?),
        file_path: row.get(3)?,
        created_at: row.get(4)?,
    })
}

/// Gets all images from the image_hashes table in the database and returns them as a vector of ImageRecord structs
///
/// Returns an empty vector if no images are found
pub fn get_images(conn: &Connection) -> Result<Vec<ImageRecord>> {
    let mut stmt =
        conn.prepare("SELECT id, crypto_hash, p_hash, file_path, created_at FROM image_hashes")?;
    let rows = stmt.query_map((), image_record_from_row)?;
    rows.collect()
}

/// Retrieves image row from database corresponding to the provided crypto hash provided
///
/// Returns `Ok(None)` if no row matches the given hash.
///
/// Failure
/// Will return Err if sql cannot be converted to a C-compatible string
/// or if the underlying SQLite call fails.
pub fn get_image_by_crypto_hash(
    conn: &Connection,
    hash: &HexString,
) -> Result<Option<ImageRecord>> {
    let result = conn.query_row(
        "SELECT id, crypto_hash, p_hash, file_path, created_at FROM image_hashes WHERE crypto_hash = ?1",
        [hash.get_hex()],
        image_record_from_row,
    );

    match result {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Checks if a comparison exists for the given image ID in the phash_comparisons table
///
/// Returns true if the id exists, false if no record is found
///
/// Failure
/// Will return Err if sql cannot be converted to a C-compatible string
/// or if the underlying SQLite call fails.
pub fn comparison_exists(conn: &Connection, id: i64) -> Result<bool> {
    conn.query_row(
        "SELECT EXISTS( SELECT 1 FROM phash_comparisons WHERE image_id_1 = ?1 OR image_id_2 = ?1)",
        [id],
        |row| row.get(0),
    )
}

/// A candidate image (id and p_hash) to compare against another image's p_hash
pub struct PHashEntry(pub i64, pub PHashHexString);

/// Retrieves all p_hashes from the image_hashes table that have not yet been compared to the given image id
///
/// Returns a vector of PHashEntry structs containing the id and p_hash of each candidate image
/// Failure
/// Err if sql cannot be converted to a C-compatible string
/// or if the underlying SQLite call fails.
pub fn get_phashes_to_compare(conn: &Connection, id: i64) -> Result<Vec<PHashEntry>> {
    let mut stmt = conn.prepare(
        "SELECT id, p_hash FROM image_hashes WHERE id != ?1 AND id NOT IN (
            SELECT image_id_2 FROM phash_comparisons WHERE image_id_1 = ?1
            UNION
            SELECT image_id_1 FROM phash_comparisons WHERE image_id_2 = ?1
        )",
    )?;
    let hashes = stmt.query_map([id], |row| {
        let other_id: i64 = row.get(0)?;
        let hash = PHashHexString::new(row.get(1)?);
        Ok(PHashEntry(other_id, hash))
    })?;

    hashes.collect()
}

/// Records the hamming distance between two images' p_hashes into the phash_comparisons table
///
/// The pair of ids is normalized to (min, max) before inserting, matching the table's
/// `CHECK (image_id_1 < image_id_2)` constraint. If the comparison already exists it is not
/// inserted again.
///
/// Failure
/// Err if sql cannot be converted to a C-compatible string
/// or if the underlying SQLite call fails.
pub fn record_comparison(
    conn: &Connection,
    image_id_1: i64,
    image_id_2: i64,
    hamming_distance: usize,
) -> Result<()> {
    let (lo, hi) = if image_id_1 < image_id_2 {
        (image_id_1, image_id_2)
    } else {
        (image_id_2, image_id_1)
    };
    conn.execute(
        "INSERT OR IGNORE INTO phash_comparisons (image_id_1, image_id_2, hamming_distance) VALUES (?1, ?2, ?3)",
        (lo, hi, hamming_distance as i64),
    )?;
    Ok(())
}

/// A struct representing a similar image entry, containing the id, file path, and hamming distance from the database
pub struct SimilarEntry(pub i64, pub String, pub i64);

/// Retrieves all image comparisons for a given image id.
///
/// Returns
/// - the id, filename, and hamming distance of the compared files, ordered by most similar
/// - an empty vector if no comparisons are found
///
/// Failure
///
/// Err if sql cannot be converted to a C-compatible string
/// or if the underlying SQLite call fails.
pub fn get_image_comparisons(
    conn: &Connection,
    image_id: i64,
    order_asc: bool,
) -> Result<Vec<SimilarEntry>> {
    let order = match order_asc {
        true => "ASC",
        false => "DESC",
    };

    let mut stmt = conn.prepare(&format!(
        "SELECT ih.id AS id, ih.file_path AS name, p.hamming_distance as hamming_distance
        FROM phash_comparisons AS p
        INNER JOIN image_hashes AS ih
           ON ih.id = CASE WHEN p.image_id_1 = ?1 THEN p.image_id_2 ELSE p.image_id_1 END
        WHERE p.image_id_1 = ?1 OR p.image_id_2 = ?1
        ORDER BY p.hamming_distance {};
        ",
        order
    ))?;

    let rows = stmt.query_map([image_id], |row| {
        let id = row.get(0)?;
        let name = row.get(1)?;
        let distance = row.get(2)?;
        Ok(SimilarEntry(id, name, distance))
    })?;

    rows.collect()
}
