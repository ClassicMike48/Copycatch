use crate::Config;
use image::DynamicImage;
use image_hasher::{self, HasherConfig};
use log::{error, info, warn};
use sha2::{Digest, Sha256};
use std::{
    fs::{self},
    io::{self, ErrorKind::InvalidData},
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq)]
pub struct HexString(String);
impl HexString {
    pub fn new(hex: String) -> Self {
        HexString(hex)
    }

    pub fn get_hex(&self) -> &String {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PHashHexString(String);
impl PHashHexString {
    pub fn new(hex: String) -> Self {
        PHashHexString(hex)
    }

    pub fn get_hex(&self) -> &String {
        &self.0
    }
}
// utilize SHA-256
// Hashes pixel data, not file data. This is to avoid false negatives when the same image is saved in different formats or with different metadata.
// Supported file format is bounded by the image crate, which includes PNG, JPEG, GIF, BMP, ICO, TIFF, and WebP. To see the latest supported files see https://docs.rs/image/latest/image/codecs/index.html#supported-formats
pub fn get_crypto_hash(image: &DynamicImage) -> HexString {
    let hash = Sha256::digest(image.as_bytes());
    let result = hex::encode(hash);
    HexString(result)
}

#[cfg(test)]
mod crypto_tests {
    use std::path::Path;

    use crate::file_walk::get_crypto_hash;
    #[test]
    fn validate_sha256_copy() {
        let image1 = image::open(Path::new("samples/duplicates/sha256test.png"))
            .expect("Image1 should open");
        let image2 = image::open(Path::new("samples/duplicates/sha256test2.png"))
            .expect("Image2 should open");
        let hash1 = get_crypto_hash(&image1);
        let hash2 = get_crypto_hash(&image2);
        assert_eq!(
            hash1, hash2,
            "These hashes should match as these files are copies"
        );
    }

    #[test]
    fn validate_sha256_different() {
        let image1 = image::open(Path::new("samples/duplicates/sha256test.png"))
            .expect("Image1 should open");
        let image2 =
            image::open(Path::new("samples/different/sha256alt.png")).expect("Image2 should open");

        let hash1 = get_crypto_hash(&image1);
        let hash2 = get_crypto_hash(&image2);
        assert_ne!(
            hash1, hash2,
            "These hashes should not match as these files are different"
        );
    }
}

// Generates a perceptual hash from the pixel image data in order to relate two images that are visually similar but not identical.
// Supported file format is bounded by the image crate, which includes PNG, JPEG, GIF, BMP, ICO, TIFF, and WebP. To see the latest supported files see https://docs.rs/image/latest/image/codecs/index.html#supported-formats
pub fn get_phash(image: &DynamicImage) -> PHashHexString {
    let hasher = HasherConfig::new()
        .preproc_dct()
        .hash_alg(image_hasher::HashAlg::Median)
        .to_hasher();
    let phash = hasher.hash_image(image);
    let hex = hex::encode(phash.as_bytes());
    PHashHexString(hex)
}

#[cfg(test)]
mod phash_tests {
    use std::path::Path;

    use crate::file_walk::{calculate_hamming_distance, get_phash};

    #[test]
    fn validate_phash_copy() {
        let image1 = image::open(Path::new("samples/duplicates/sha256test.png"))
            .expect("Image1 should open");
        let image2 = image::open(Path::new("samples/duplicates/sha256test2.png"))
            .expect("Image2 should open");

        let hash1 = get_phash(&image1);
        let hash2 = get_phash(&image2);
        assert_eq!(
            hash1, hash2,
            "These hashes should match as these files are copies"
        );
        assert_eq!(
            calculate_hamming_distance(&hash1, &hash2).expect("Hashes should be comparable"),
            0,
            "Identical images should have a hamming distance of 0"
        );
    }

    #[test]
    fn validate_phash_near_duplicate() {
        // These fixtures are the same flower image shifted by a few pixels: good
        // Assert the tolerance instead of asserting the hashes differ.
        let image1 =
            image::open(Path::new("samples/different/sha256test.png")).expect("Image1 should open");
        let image2 =
            image::open(Path::new("samples/different/sha256alt.png")).expect("Image2 should open");
        let hash1 = get_phash(&image1);
        let hash2 = get_phash(&image2);
        let distance =
            calculate_hamming_distance(&hash1, &hash2).expect("Hashes should be comparable");
        assert!(
            distance <= 8,
            "A minor positional shift should still be perceptually near-identical, got hamming distance {}",
            distance
        );
    }

    #[test]
    fn different_files() {
        let image1 =
            image::open(Path::new("samples/different/sha256test.png")).expect("Image1 should open");
        let image2 =
            image::open(Path::new("samples/different/unique.png")).expect("Image2 should open");
        let hash1 = get_phash(&image1);
        let hash2 = get_phash(&image2);
        let distance =
            calculate_hamming_distance(&hash1, &hash2).expect("Hashes should be comparable");
        assert!(
            distance > 16,
            "These files are different and should have a hamming distance greater than 16, got {}",
            distance
        );
    }
}

fn calculate_hamming_distance(hash1: &PHashHexString, hash2: &PHashHexString) -> io::Result<usize> {
    let byte1 = match hex::decode(hash1.get_hex()) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Error while decoding hash1. {}", e),
            ));
        }
    };

    let byte2 = match hex::decode(hash2.get_hex()) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Error while decoding hash2. {}", e),
            ));
        }
    };

    if byte1.len() != byte2.len() {
        return Err(io::Error::new(
            InvalidData,
            "hashes are not the same length so hamming distance is invalid",
        ));
    }

    let diff_count = byte1
        .iter()
        .zip(byte2.iter())
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum();
    Ok(diff_count)
}

// Compares image_id's p_hash against every not-yet-compared image in the database and
// persists each resulting hamming distance. Failures are logged and tracked via config.stats
// rather than propagated, matching the rest of analyze_folder's error handling.
fn record_comparisons(config: &mut Config, image_id: i64, phash: &PHashHexString) {
    let entries = match crate::db::get_phashes_to_compare(&config.conn, image_id) {
        Ok(entries) => entries,
        Err(e) => {
            warn!(
                "Failed to fetch comparison candidates for image {}: {}",
                image_id, e
            );
            config.stats.database_errors += 1;
            return;
        }
    };

    for crate::db::PHashEntry(other_id, other_phash) in entries {
        let distance = match calculate_hamming_distance(phash, &other_phash) {
            Ok(distance) => distance,
            Err(e) => {
                warn!(
                    "Failed to compare image {} against {}: {}",
                    image_id, other_id, e
                );
                config.stats.comparison_errors += 1;
                continue;
            }
        };

        match crate::db::record_comparison(&config.conn, image_id, other_id, distance) {
            Ok(_) => {
                info!(
                    "Recorded comparison between {} and {} with distance {}",
                    image_id, other_id, distance
                );
                config.stats.comparisons_performed += 1;
            }
            Err(e) => {
                warn!(
                    "Failed to record comparison between {} and {}: {}",
                    image_id, other_id, e
                ); 
                config.stats.database_errors += 1;
            }
        }
    }
}

// Runs comparisons for every image currently in the database. Used when config.compare_on_run
// is false, so comparisons are deferred until after the whole folder walk has finished instead
// of running inline as each file is recorded.
pub fn compare_all_images(config: &mut Config) {
    let images = match crate::db::get_images(&config.conn) {
        Ok(images) => images,
        Err(e) => {
            warn!("Failed to fetch images for deferred comparison: {}", e);
            config.stats.database_errors += 1;
            return;
        }
    };

    for image in images {
        record_comparisons(config, image.id, &image.p_hash);
    }
}

#[cfg(test)]
mod hamming_distance_tests {
    use std::path::Path;

    use crate::file_walk::{PHashHexString, get_phash};

    use super::calculate_hamming_distance;
    #[test]
    fn matching_hash() {
        let path = Path::new("samples/different/unique.png");
        let image1 = image::open(path).expect("Image1 should open");
        let hash1 = get_phash(&image1);
        let hash2 = get_phash(&image1);
        let distance =
            calculate_hamming_distance(&hash1, &hash2).expect("Hashes should be comparable");
        assert_eq!(distance, 0);
    }
    #[test]
    fn exact_count() {
        let hash1 = PHashHexString(hex::encode("00001000"));
        let hash2 = PHashHexString(hex::encode("00000000"));
        let distance =
            calculate_hamming_distance(&hash1, &hash2).expect("Hashes should be comparable");
        assert_eq!(distance, 1);
    }
}

pub fn analyze_folder(dir_path: &Path, config: &mut Config) -> io::Result<()> {
    // Recursive calls below only ever pass paths the loop has already confirmed are
    // real (non-symlink) directories, so this mainly guards the initial call.
    // Metadata is queried to determine if the path is a directory or a symlink.
    // If the path is not a directory, an error will be returned.
    // Follows symlinks so a symlinked root is scanned like a normal directory.
    let metadata = match fs::metadata(dir_path) {
        Ok(metadata) => metadata,
        Err(e) => {
            error!("Could not stat {}: {}", dir_path.display(), e);
            return Err(e);
        }
    };

    if !metadata.is_dir() {
        error!("Path provided is not a directory: {}", dir_path.display());
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!("Path provided is not a directory: {}", dir_path.display()),
        ));
    }

    // Init file iterator and catch any directory-like errors (i.e folder permission errors)
    let entries = match fs::read_dir(dir_path) {
        Ok(entries) => entries,
        Err(e) => {
            // Pass on the error created by read_dir to the caller, log it at Error level as the entire directory will be skipped.
            error!("{}", e);
            return Err(e);
        }
    };

    // Iterate over all the files within the directory
    for entry in entries {
        let entry = match entry {
            Ok(file) => file,
            Err(e) => {
                // Document error and continue to next entry
                warn!("{}", e);
                config.stats.file_read_error += 1;
                continue;
            }
        };

        // File could be read, continue processing
        // Check if the current entry is a file or a directory.
        // Directory - Keep scanning for more entries in the file tree
        // File - Check if it's a compatible image for processing
        // Directory check is still necessary here as this function is called recursively and the initial call needs to be checked prior to traversal.
        let path = entry.path();

        // Use the entry's file type (sourced from readdir, not a followed stat) so
        // symlinks are detected without resolving them, and so permission/I-O errors
        // surface as an explicit error instead of being silently treated as "not a directory".

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(e) => {
                warn!(
                    "Could not determine file type for {}: {}",
                    path.display(),
                    e
                );
                config.stats.file_read_error += 1;
                continue;
            }
        };

        // Continue to next entry if current entry is a symlink that points to a directory. This prevents cycles in the file system from crashing the program.
        if file_type.is_symlink() {
            match fs::metadata(&path) {
                Ok(target_metadata) if target_metadata.is_dir() => {
                    //symlink resolves to a directory, do not follow and update stats
                    config.stats.symlinks_skipped += 1;
                    warn!(
                        "Skipping symlink {} that points to a directory",
                        path.display()
                    );
                    continue;
                }
                Ok(_) => {
                    //symlink resolves to a file - fall through
                    info!(
                        "Symlink {} points to a file. Allowing backup... ",
                        path.display()
                    );
                    config.stats.symlinks_allowed += 1;
                }
                Err(e) => {
                    config.stats.symlinks_skipped += 1;
                    warn!(
                        "Skipping symlink {} as target metadata could not be read: {}",
                        path.display(),
                        e
                    );
                    continue;
                }
            }
        }
        if file_type.is_dir() {
            // Recur through the rest of the file tree
            config.stats.directories_found += 1;
            // If an error occurs while analyzing the folder, log it and continue to the next entry.
            if let Err(e) = analyze_folder(&path, config) {
                error!("{}", e);
            };
        } else {
            // Entry is a file
            config.stats.files_seen += 1;
            // Check for an image file and operate
            let image = match image::open(&path) {
                Ok(image) => image,
                Err(e) => {
                    //skip non-image files
                    warn!("Failed to open image file {}: {}", path.display(), e);
                    continue;
                }
            };
            config.stats.image_files_seen += 1;

            let crypto_hash = get_crypto_hash(&image);

            // Supported image file detected, update stats
            match crate::db::crypto_hash_exists(&config.conn, &crypto_hash) {
                Ok(true) => {
                    info!("Duplicate detected: {}", path.display());
                    config.stats.duplicates_detected += 1;
                    continue;
                }
                Ok(false) => {}
                Err(e) => {
                    warn!("Failed to get hash record for {}: {}", path.display(), e);
                    config.stats.database_errors += 1;
                    continue;
                }
            }

            // Compute the perceptual hash - Storing this hash helps with image comparison features
            let phash = get_phash(&image);

            // Not a duplicate photo that has been previously seen,
            // Including across previous runs of the program (see db module)
            let backup_file_name = match backup_file(&path, &config.backup_location) {
                Ok(path) => path,
                Err(e) => {
                    warn!("{} could not be copied: {}", path.display(), e);
                    config.stats.copy_errors += 1;
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
            match crate::db::record_file(&config.conn, &crypto_hash, &phash, &backup_file_name_str)
            {
                Ok(_) => {
                    config.stats.image_files_copied += 1;
                    if config.compare_on_run {
                        match crate::db::get_image_by_crypto_hash(&config.conn, &crypto_hash) {
                            Ok(Some(record)) => record_comparisons(config, record.id, &phash),
                            Ok(None) => warn!(
                                "Recorded file {} but could not find it immediately after insert",
                                path.display()
                            ),
                            Err(e) => {
                                warn!("Failed to look up recorded file {}: {}", path.display(), e);
                                config.stats.database_errors += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to persist hash record for {}: {}",
                        path.display(),
                        e
                    );
                    config.stats.database_errors += 1;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod analyze_folder_tests {
    use std::path::{Path, PathBuf};

    use crate::file_walk::analyze_folder;
    use crate::{Config, Stats};

    #[test]
    fn skips_non_image_file_without_error() {
        let conn =
            crate::db::init(Path::new(":memory:")).expect("in-memory db should initialize");
        let mut config = Config {
            // backup_file() is never invoked on this path, so this location is never touched.
            backup_location: PathBuf::new(),
            compare_on_run: false,
            conn,
            stats: Stats::default(),
        };

        analyze_folder(Path::new("samples/non_image"), &mut config)
            .expect("walking a directory of only non-image files should not error");

        assert_eq!(config.stats.files_seen, 1, "the .txt file should be counted as seen");
        assert_eq!(
            config.stats.image_files_seen, 0,
            "a non-image file must not be counted as an image"
        );
        assert_eq!(config.stats.database_errors, 0);

        let images = crate::db::get_images(&config.conn).expect("query should succeed");
        assert!(
            images.is_empty(),
            "no image record should have been created for a non-image file"
        );
    }
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
    std::fs::copy(source_path, &destination_path)?;
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
pub fn validate_save_location(dir_path: &Path) -> Result<(), std::io::Error> {
    match dir_path.try_exists() {
        Ok(found) => {
            if found {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Directory does not exist: {}", dir_path.display()),
                ))
            }
        }
        Err(e) => {
            error!("Issues verifying backup folder: {}", dir_path.display());
            Err(e)
        }
    }
}
