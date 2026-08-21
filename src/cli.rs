use crate::config::{Config, Stats};
use crate::file_walk::{analyze_folder, compare_all_images, validate_save_location};
use clap::{Args, Parser, Subcommand};
use log::{error, info};
use std::io::ErrorKind::NotFound;
use std::path::{Path, PathBuf};

/// A command line tool to analyze and manage image files in a centralized location. It scans directories for image files, copies them to a backup location, and compares them for similarities.
#[derive(Parser, Debug)]
#[command(version="0.1", about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Commands {
    /// Run p-hash comparisons of all items in the database
    Compare(CompareArgs),
    /// Backup files and build metadata
    Backup(BackupArgs),
}

#[derive(Args, Debug)]
pub(crate) struct CompareArgs {
    /// Location of folder backup containing the image database file to update
    #[arg(short, long, default_value = "backup/")]
    destination_folder: String,
}

impl CompareArgs {
    /// Runs the compare command with the provided arguments.
    pub(crate) fn run(&self) -> () {
        // verify that the provided destination will work or utilize sensible defaults.
        let backup_location = handle_save_location(&self.destination_folder).unwrap();

        // Initialize Config
        // Check if the database file exists at the specified location
        let db_path = backup_location.join("photo_manager.db");
        validate_db_location(&db_path).unwrap();
        let conn = crate::db::init(&db_path)
            .expect("Database should exist and initialize successfully");
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

fn validate_db_location(db_path: &Path) -> Result<(), std::io::Error> {
    match db_path.try_exists() {
        Ok(true) => { return Ok(())}
        Ok(false) => {
            error!(
                "Database file does not exist at path: {}",
                db_path.display()
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Database file does not exist",
            ));
        }
        Err(e) => {
            error!("Error checking database file existence: {}", e);
            return Err(e);
        }
    }
}

#[derive(Args, Debug)]
pub(crate) struct BackupArgs {
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
    pub(crate) fn run(&self) -> () {
        // verify that the provided destination will work or utilize sensible defaults.
        let backup_location = handle_save_location(&self.destination_folder).unwrap();
        let db_path = backup_location.join("photo_manager.db");
        let conn = crate::db::init(&db_path).expect("Failed to initialize database");
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
    //handle return
    match validate_save_location(Path::new(&backup_location)) {
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
    }
}
