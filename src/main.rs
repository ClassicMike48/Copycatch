use std::env;
use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};
use image::DynamicImage;
use sha2::{Digest, Sha256};

// create a file walker that will visit all files in a tree
//recursive function
fn print_file_name(file: &DirEntry) {
    println!("{}", file.file_name().display());
}

//TODO consider adding a cryptographic hash for exact file detection
//utilize SHA-256 
fn get_crypto_hash(image: &DynamicImage) -> String {
    let data = image.as_bytes();
    let hash = Sha256::digest(data);
    hash.iter().map(|b| format!("{:02x}", b)).collect()
}
fn get_phash(file: &DirEntry) {
    let path = file.path();
    match image::open(&path) {
        Ok(image) => {
            let phash = imagehash::perceptual_hash(&image);
            phash.to_string();
            let sha = get_crypto_hash(&image);
            println!("{}\n- phash: {}\n- sha256: {}", path.display(), phash, sha);
        }
        Err(_) => println!(
            "Error occurred while trying to read from {}",
            path.display()
        ),
    }
}

fn calculate_hamming_distance(hash1: &imagehash::Hash, hash2: &imagehash::Hash) -> usize {
    let bits1 = &hash1.bits;
    let bits2 = &hash2.bits;
    
    let count = bits1.iter().zip(bits2.iter()).filter(|(a, b)| a != b).count();
    return count;
}

fn visit_directory(dir_path: &Path, action: &dyn Fn(&DirEntry)) -> io::Result<()> {
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
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() < 2 {
        // No directory provided, use default
        println!("No path provided, using default directory: 'test'");
        let _ = visit_directory(Path::new("test"), &get_phash);
    } else {
        let dir_path = &args[1];
        let _ = visit_directory(Path::new(dir_path), &print_file_name);
    }
}
//Hash outputs in lowercase hex
// 50da58893c4db448c810d689fc9f4ef9e091d37f5aab4e383a1142e287e4f8bf
// 000036dadbdbdbda