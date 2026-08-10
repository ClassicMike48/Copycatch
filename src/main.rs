use crate::file_walk::{analyze_folder, compare_all_images, validate_save_location};
use clap::Parser;
use log::LevelFilter;
use log::info;
use rusqlite::Connection;
use std::backtrace;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
mod db;
mod file_walk;

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

struct Config {
    backup_location: PathBuf,
    compare_on_run: bool,
    conn: Connection,
    stats: Stats,
}

/// A command line tool to analyze and manage image files in a centralized location. It scans directories for image files, copies them to a backup location, and compares them for similarities.
#[derive(Parser, Debug)]
#[command(version="0.1", about, long_about = None)]
struct Args {
    /// Location to create image database and store image backups. If not provided, defaults to 'backup/'.
    #[arg(short, long, default_value = "backup/")]
    destination_folder: String,

    /// Generate p_hashs comparison data on images found during the backup process. If not provided, defaults to false.
    #[arg(short, long, default_value = "false")]
    compare_on_run: bool,

    /// Location to scan for image files to backup.
    #[arg(short, long)]
    source_folder: String,
}
fn main() {
    //Default location to store file backups -- prompt user
    let args = Args::parse();
    //Setup program logger
    env_logger::builder()
        .format(|buff, record| writeln!(buff, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Auto) //For security, logging user input should be sanitized or use WriteStyle::never
        .init();

    //let args: Vec<String> = env::args().collect();
    println!("{:#?}", args);

    let destination_path = match args.destination_folder.as_str() {
        "backup/" => {
            info!("No path provided, using default directory: 'backup/'");
            "backup/"
        }
        location => {
            info!("Backing up files to {}", location);
            location
        }
    };
    let backup_location = Path::new(&destination_path).to_path_buf();
    validate_save_location(Path::new(&backup_location)).expect("Failed to validate save location");

    let db_path = backup_location.join("photo_manager.db");
    let conn = db::init(&db_path).expect("Failed to initialize database");
    let stats = Stats::default();
    let compare_on_run = args.compare_on_run;

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
