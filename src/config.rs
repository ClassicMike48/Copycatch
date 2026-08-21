use rusqlite::Connection;
use std::path::PathBuf;

#[derive(Default, Debug)]
pub(crate) struct Stats {
    pub(crate) directories_found: u64,
    pub(crate) files_seen: u64,
    pub(crate) file_read_error: u64,
    pub(crate) image_files_copied: u64,
    pub(crate) copy_errors: u64,
    pub(crate) duplicates_detected: u64,
    pub(crate) image_files_seen: u64,
    pub(crate) database_errors: u64,
    pub(crate) symlinks_skipped: u64,
    pub(crate) symlinks_allowed: u64,
    pub(crate) comparisons_performed: u64, // phash comparisons
    pub(crate) comparison_errors: u64,
}

pub(crate) struct Config {
    pub(crate) backup_location: PathBuf,
    pub(crate) compare_on_run: bool,
    pub(crate) conn: Connection,
    pub(crate) stats: Stats,
}
