use crate::file_walk::{analyze_folder, compare_all_images, validate_save_location};
use log::LevelFilter;
use log::info;
use rusqlite::Connection;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
mod db;
mod file_walk;

struct Config {
    backup_location: PathBuf,
    compare_on_run: bool,
    conn: Connection,
    stats: Stats,
}

#[derive(Default, Debug)]
struct Stats {
    directories_found: u64,
    files_seen: u64,
    file_read_error: u64,
    image_files_copied: u64,
    copy_errors: u64,
    duplicates_detected: u64,
    image_files_seen: u64,
    database_errors: u64,
    symlinks_skipped: u64,
    symlinks_allowed: u64,
    comparison_errors: u64,
}
fn main() {
    //Default location to store file backups -- prompt user

    //Setup program logger
    env_logger::builder()
        .format(|buff, record| writeln!(buff, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Auto) //For security, logging user input should be sanitized or use WriteStyle::never
        .init();

    let args: Vec<String> = env::args().collect();
    println!("{:#?}", args);
    if args.len() < 2 {
        // No directory provided, use default
        info!("No path provided, using default directory: 'test'");
        let backup_location = Path::new("backup/").to_path_buf();
        info!("{}", backup_location.is_dir());

        //Validate the location for copying files
        validate_save_location(&backup_location).unwrap();

        let db_path = backup_location.join("photo_manager.db");
        let conn = db::init(&db_path).expect("Failed to initialize database");
        let stats = Stats::default();
        let compare_on_run = false;
        let mut config = Config {
            backup_location,
            conn,
            stats,
            compare_on_run,
        };
        analyze_folder(Path::new("test"), &mut config).unwrap();

        if !config.compare_on_run {
            info!("Starting image comparisons for similarities");
            compare_all_images(&mut config);
        }

        info!("Displaying results of search...");
        println!("{:#?}", config.stats);
    }
}
