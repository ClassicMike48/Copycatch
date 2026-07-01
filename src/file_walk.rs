use image::{DynamicImage, ImageError};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};

// create a file walker that will visit all files in a tree
//recursive function
pub fn print_file_name(file: &DirEntry) {
    println!("{}", file.file_name().display());
}

//TODO consider adding a cryptographic hash for exact file detection
//utilize SHA-256
// pub fn get_crypto_hash(image: &DynamicImage) -> String {
//     let data = image.as_bytes();
//     let hash = Sha256::digest(data);
//     hash.iter().map(|b| format!("{:02x}", b)).collect()
// }

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
            println!(
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
            println!(
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
