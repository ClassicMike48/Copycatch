use std::env;
use std::path::{Path, PathBuf};
use crate::file_walk::{
    analyze_folder, compare_all_images, validate_save_location,
};
use log::LevelFilter;
use log::info;
use rusqlite::Connection;
use std::io::Write;
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
    total_directories_found: u64,
    total_files_seen: u64,
    total_file_read_error: u64,
    total_image_files_seen: u64,
    total_image_files_copied: u64,
    total_copy_errors: u64,
    total_duplicates_detected: u64,
    total_database_errors: u64,
    total_symlinks_skipped: u64,
    total_symlinks_allowed: u64,
    total_comparison_errors: u64,
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
    } else {
        let _dir_path = &args[1];

        let a = b'1';
        let b = b'1';
        println!("{} {} {}", a, b, (a ^ b).count_ones());
        // let dir = read_dir("test/sample1/").expect("Should work");
        // for file in dir {
        //     if let Ok(file) = file {
        //         if let Ok(hash) = get_phash(&file) {
        //             println!("File: {:?}, Hash: {:?}", file.path(), hash.as_bytes());
        //         };
        //     }
        // }
    }
}

//Hash outputs in lowercase hex
// 50da58893c4db448c810d689fc9f4ef9e091d37f5aab4e383a1142e287e4f8bf
// 000036dadbdbdbda
