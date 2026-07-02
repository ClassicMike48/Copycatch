use std::env;
use std::path::Path;
use crate::file_walk::{get_phash,print_file_name,visit_directory};


mod file_walk;
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:?}", args);
    if args.len() < 2 {
        // No directory provided, use default
        println!("No path provided, using default directory: 'test'");
        visit_directory(Path::new("test"), &get_phash);
    } else {
        let dir_path = &args[1];
        visit_directory(Path::new(dir_path), &print_file_name);
    }
}
//Hash outputs in lowercase hex
// 50da58893c4db448c810d689fc9f4ef9e091d37f5aab4e383a1142e287e4f8bf
// 000036dadbdbdbda