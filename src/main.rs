use std::path::{PathBuf, Path};
use std::{collections::HashMap, env};

use crate::file_walk::{analyze_folder, print_file_name, visit_directory};
use log::LevelFilter;
use log::info;
use std::io::Write;
mod file_walk;

struct Config {
    images_map: HashMap<String, Vec<String>>,
    backup_location: PathBuf,
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
        let mut config = Config {
            images_map: HashMap::new(),
            backup_location: Path::new("backup/").to_path_buf(),
        };
        println!("{}", config.backup_location.is_dir());
        analyze_folder(Path::new("test"), &mut config).unwrap();

        info!("Displaying results of search...");
        for (k, v) in config.images_map.iter() {
            println!("Hash: {} - Files: [{}]", k, v.join(", "))
        }
    } else {
        let dir_path = &args[1];
        visit_directory(Path::new(dir_path), &print_file_name).unwrap();
    }
}

//Hash outputs in lowercase hex
// 50da58893c4db448c810d689fc9f4ef9e091d37f5aab4e383a1142e287e4f8bf
// 000036dadbdbdbda
