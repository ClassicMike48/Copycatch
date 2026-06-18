use image::{ImageFormat, ImageReader};
use std::env;
use std::{
    fs::{self, DirEntry},
    io,
    path::Path,
};
// create a file walker that will visit all files in a tree
//recursive function

fn print_file_name(file: &DirEntry) {
    println!("{}", file.file_name().display());
}

fn print_path(file: &DirEntry) {
    let path = file.path();
    if let Ok(image) = ImageReader::open(&path) {
        if let Ok(image) = image.decode() {
            //Process raw image
        } else {
            println!("{} not a supported image type.", file.file_name().display());
            print!(
                "- Supported types include [Png, Jpeg, Gif, WebP, Pnm, Tiff, Tga, Dds, Bmp, Ico, Hdr, OpenExr, Farbfeld, Avif, Qoi]",
            )
        }
    } else {
        println!("Error occurred while trying to read from {}", path.display())
    }
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
        let _ = visit_directory(Path::new("test"), &print_path);
    } else {
        let dir_path = &args[1];
        let _ = visit_directory(Path::new(dir_path), &print_file_name);
    }
}
