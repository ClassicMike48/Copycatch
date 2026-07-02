use std::path::Path;
use std::{collections::HashMap, env};

use crate::file_walk::{analyze_folder, print_file_name, visit_directory};

mod file_walk;
fn main() {
    let args: Vec<String> = env::args().collect();
    println!("{:#?}", args);
    if args.len() < 2 {
        // No directory provided, use default
        println!("No path provided, using default directory: 'test'");
        // visit_directory(Path::new("test"), &get_phash);
        let mut pic_map = HashMap::new();
        analyze_folder(Path::new("test"), &mut pic_map).unwrap();
        // println!("{:?}", pic_map);
        for (k, v) in pic_map.drain() {
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
