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
        let _ = visit_directory(Path::new("test"), &print_file_name);
    } else {
        let dir_path = &args[1];
        let _ = visit_directory(Path::new(dir_path), &print_file_name);
    }
}
