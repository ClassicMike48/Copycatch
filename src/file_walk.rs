use crate::Config;
use image::ImageError;
use log::{error, info, warn};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, DirEntry},
    io::{self},
    path::{Path, PathBuf},
};

// create a file walker that will visit all files in a tree
// recursive function
pub fn print_file_name(file: &DirEntry) {
    println!("{}", file.file_name().display());
}

// TODO consider adding a cryptographic hash for exact file detection
// utilize SHA-256

pub fn get_crypto_hash(file: &DirEntry) -> Result<String, ImageError> {
    let path = file.path();
    match image::open(&path) {
        Ok(image) => {
            let data = image.as_bytes();
            let hash = Sha256::digest(data);
            let result = hash.iter().map(|b| format!("{:02x}", b)).collect();
            Ok(result)
        }
        Err(e) => {
            warn!(
                "Error occurred while trying to read from {}",
                path.display()
            );
            Err(e)
        }
    }
}
pub fn get_phash(file: &DirEntry) -> Result<String, ImageError> {
    let path = file.path();
    match image::open(&path) {
        Ok(image) => {
            let phash = imagehash::perceptual_hash(&image);
            Ok(phash.to_string())
        }
        Err(e) => {
            warn!(
                "Error occurred while trying to read from {}",
                path.display()
            );
            Err(e)
        }
    }
}

fn calculate_hamming_distance(hash1: &imagehash::Hash, hash2: &imagehash::Hash) -> usize {
    let bits1 = &hash1.bits;
    let bits2 = &hash2.bits;

    let count = bits1
        .iter()
        .zip(bits2.iter())
        .filter(|(a, b)| a != b)
        .count();
    return count;
}

// Walk file system and attempt to perform action on said file.
pub fn visit_directory(dir_path: &Path, action: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    visit_directory(&path, action)?;
                } else {
                    action(&entry);
                }
            }
        }
    }
    Ok(())
}

pub fn analyze_folder(dir_path: &Path, config: &mut Config) -> io::Result<()> {
    // Check that path provided is a directory, mostly used to verify that @param dir_path is a directory when starting the file walk.
    if dir_path.is_dir() {
        // Iterate over all the files within the directory
        let entries = match fs::read_dir(dir_path) {
            Ok(entries) => entries,
            Err(e) => {
                error!("{}", e);
                return Err(e);
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(file) => file,
                Err(e) => {
                    //Document error and continue to next entry
                    warn!("{}", e);
                    config.stats.total_file_read_error += 1;
                    continue;
                }
            };

            // File could be read, continue processing
            // Check if the current entry is a file or a directory.
            // Directory - Keep scanning for more entries in the file tree
            // File - Check if it's a compatible image for processing
            let path = entry.path();
            if path.is_dir() {
                //recur through the rest of the file tree
                config.stats.total_directories_found += 1;
                if let Err(e) = analyze_folder(&path, config) {
                    error!("{}", e);
                };
            } else {
                // entry is a file
                config.stats.total_files_seen += 1;
                // check for an image file and operate
                if let Ok(hash) = get_crypto_hash(&entry) {
                    //supported image file detected, update stats
                    config.stats.total_image_files_seen += 1;
                    match crate::db::hash_exists(&config.conn, &hash) {
                        Ok(true) => {
                            info!("Duplicate detected: {}", path.display());
                            config.stats.total_duplicates_detected += 1;
                            continue;
                        }
                        Ok(false) => {}
                        Err(e) => {
                            warn!("Failed to get hash record for {}: {}", path.display(), e);
                            config.stats.total_database_errors += 1;
                            continue;
                        }
                    }

                    // Not a duplicate photo that has been previously seen,
                    // including across previous runs of the program (see db module)
                    let backup_file_name = match backup_file(&path, &config.backup_location) {
                        Ok(path) => path,
                        Err(e) => {
                            warn!("{} could not be copied: {}", path.display(), e);
                            config.stats.total_copy_errors += 1;
                            continue;
                        }
                    };
                    // Persist so this hash is known on subsequent runs
                    let backup_file_name_str = backup_file_name
                        .file_name()
                        .unwrap() //.file_name() should not return None here will how backup_file_name is generated. If this panics, the process of naming the file should be inspected.
                        .to_string_lossy()
                        .to_string();

                    //TODO Consider the order of copying and commiting to database and whether this is the best order.
                    match crate::db::record_file(&config.conn, &hash, &backup_file_name_str) {
                        Ok(_) => config.stats.total_image_files_copied += 1,
                        Err(e) => {
                            warn!(
                                "Failed to persist hash record for {}: {}",
                                path.display(),
                                e
                            );
                            config.stats.total_database_errors += 1;
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn backup_file(source_path: &Path, destination_folder: &Path) -> io::Result<PathBuf> {
    let Some(original_file_name) = source_path.file_name() else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "Could not determine file name for {}",
                source_path.display()
            ),
        ));
    };
    let destination_path = unique_backup_path(destination_folder, original_file_name);
    std::fs::copy(&source_path, &destination_path)?;
    Ok(destination_path)
}

// If a file with the same name already exists in the backup folder (but isn't a
// hash duplicate), disambiguate by appending a short number suffix to the name.
fn unique_backup_path(backup_dir: &Path, original_file_name: &std::ffi::OsStr) -> PathBuf {
    let candidate = backup_dir.join(original_file_name);
    if !candidate.exists() {
        return candidate;
    }

    // Parse file name into various components
    let name_path = Path::new(original_file_name);
    let stem = name_path //filename without extension
        .file_stem()
        .unwrap_or(original_file_name)
        .to_string_lossy();
    let extension = name_path.extension().map(|ext| ext.to_string_lossy());

    // Keep generating suffix until a unique one is produced
    for n in 1.. {
        let candidate = match &extension {
            Some(ext) => backup_dir.join(format!("{}_{}.{}", stem, n, ext)),
            None => backup_dir.join(format!("{}_{}", stem, n)),
        };
        if !candidate.exists() {
            return candidate;
        }
    }
    unreachable!()
}

// Encapsulate runtime checking of backup folder existing and available for operations
// Can be later expanded to check for previous program save states, and improved error messaging.
pub fn validate_save_location(dir_path: &PathBuf) -> Result<(), std::io::Error> {
    match dir_path.try_exists() {
        Ok(found) => {
            if found {
                return Ok(());
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Directory does not exist: {}", dir_path.display()),
                ));
            }
        }
        Err(e) => {
            error!("Issues verifying backup folder: {}", dir_path.display());
            return Err(e);
        }
    }
}
