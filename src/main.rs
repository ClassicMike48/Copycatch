use crate::file_walk::{analyze_folder, compare_all_images, validate_save_location};
use clap::{Args, Parser, Subcommand};
use log::LevelFilter;
use log::{error, info, warn};
use rusqlite::Connection;
use std::io::ErrorKind::NotFound;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};
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
    comparisons_performed: u64, // phash comparisons
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
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run p-hash comparisons of all items in the database
    Compare(CompareArgs),
    /// Backup files and build metadata
    Backup(BackupArgs),
}

#[derive(Args, Debug)]
struct CompareArgs {
    /// Location of folder backup containing the image database file to update
    #[arg(short, long, default_value = "backup/")]
    destination_folder: String,
}

impl CompareArgs {
    /// Runs the compare command with the provided arguments.
    fn run(&self) -> () {
        // verify that the provided destination will work or utilize sensible defaults.
        let backup_location = handle_save_location(&self.destination_folder).unwrap();

        // Initialize Config
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

        compare_all_images(&mut config);
        // print results
        info!("Displaying results of comparisons...");
        println!("{:#?}", config.stats);
    }
}

#[derive(Args, Debug)]
struct BackupArgs {
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

impl BackupArgs {
    /// Runs the backup command with the provided arguments.
    fn run(&self) -> () {
        // verify that the provided destination will work or utilize sensible defaults.
        let backup_location = handle_save_location(&self.destination_folder).unwrap();

        let db_path = backup_location.join("photo_manager.db");
        let conn = db::init(&db_path).expect("Failed to initialize database");
        let stats = Stats::default();
        let compare_on_run = self.compare_on_run;

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

/// Helper function that verifies that the provided save location is valid and returns a PathBuf. Will panic if the save location is invalid.
fn handle_save_location(path: &str) -> Result<PathBuf, std::io::Error> {
    let destination_path = match path {
        "backup/" => {
            info!("No path provided, using default directory: 'backup/'");
            "backup/"
        }
        location => {
            info!("Custom path provided: {}", location);
            location
        }
    };
    let backup_location = Path::new(&destination_path).to_path_buf();
    let result = match validate_save_location(Path::new(&backup_location)) {
        Ok(_) => Ok(backup_location),
        Err(e) => {
            if e.kind() == NotFound && destination_path == "backup/" {
                info!(
                    "Default backup location does not exist, creating directory: {}",
                    backup_location.display()
                );

                Err(e)
            } else {
                error!("Error validating save location: {}", e);
                Err(e)
            }
        }
    };
    result
}

fn main() {
    // //Default location to store file backups -- prompt user
    let args = Cli::parse();

    //Setup program logger
    env_logger::builder()
        .format(|buff, record| writeln!(buff, "{}: {}", record.level(), record.args()))
        .filter(None, LevelFilter::Info)
        .write_style(env_logger::WriteStyle::Auto) //For security, logging user input should be sanitized or use WriteStyle::never
        .init();

    println!("{:#?}", args);
    match &args.command {
        Commands::Compare(compare_args) => {
            compare_args.run();
            return;
        }
        Commands::Backup(backup_args) => {
            backup_args.run();
            return;
        }
        _ => {}
    }
}
