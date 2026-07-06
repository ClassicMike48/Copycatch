use crate::Config;
use image::ImageError;
use log::{error, info, warn};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    io::{self, Error},
    path::{Path, PathBuf},
};

// create a file walker that will visit all files in a tree
//recursive function
pub fn print_file_name(file: &DirEntry) {
    println!("{}", file.file_name().display());
}

//TODO consider adding a cryptographic hash for exact file detection
//utilize SHA-256

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

//Walk file system and attempt to perform action on said file.
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
    if dir_path.is_dir() {
        for entry in fs::read_dir(dir_path)? {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    //recur through the rest of the file tree
                    analyze_folder(&path, config)?;
                } else {
                    //entry is a file
                    //check for an image file and operate
                    if let Ok(hash) = get_crypto_hash(&entry) {
                        let images = config.images_map.entry(hash.clone()).or_insert(Vec::new());
                        //Debugging
                        if images.len() > 0 {
                            info!("Duplicate detected: {}", path.display());
                        } else {
                            //Not a duplicate photo that has been previously seen
                            //TODO Current setup doesn't factor in previous runs of the program. This will lead to duplications unless the hashmap is updated prior to rerunning. Further pushing the importance of a database.
                            backup_file(&path, &config.backup_location)?;
                        }
                        //Document the hash value, even if its a duplicate
                        images.push(path.display().to_string());
                    }
                }
            }
        }
    }
    Ok(())
}

fn backup_file(source_path: &Path, destination_folder: &Path) -> io::Result<()> {
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
    std::fs::copy(source_path, destination_path)?;
    Ok(())
}
// If a file with the same name already exists in the backup folder (but isn't a
// hash duplicate), disambiguate by appending a short number suffix to the name.
fn unique_backup_path(backup_dir: &Path, original_file_name: &std::ffi::OsStr) -> PathBuf {
    let candidate = backup_dir.join(original_file_name);
    if !candidate.exists() {
        return candidate;
    }

    //Parse file name into various components
    let name_path = Path::new(original_file_name);
    let stem = name_path //filename without extension
        .file_stem()
        .unwrap_or(original_file_name)
        .to_string_lossy();
    let extension = name_path.extension().map(|ext| ext.to_string_lossy());

    //Keep generating suffix until a unique one is produced
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

//encapsulate runtime checking of backup folder existing and available for operations
//Can be later expanded to check for previous program save states, and improved error messaging.
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
