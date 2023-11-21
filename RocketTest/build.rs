use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Set the output directory where the binary will be placed after build
    let out_dir = env::var("OUT_DIR").expect("Unable to get OUT_DIR");

    // Path to the templates directory
    let source = Path::new("templates");

    // Path to the directory where the binary will be placed after build
    let target = Path::new(&out_dir).join("../../../").join("templates");

    // Copy the 'templates' directory to the output directory
    if let Err(err) = fs::create_dir_all(&target) {
        eprintln!("Failed to create directory: {}", err);
    }

    if let Err(err) = fs::copy(&source, &target) {
        eprintln!("Failed to copy 'templates' directory: {}", err);
    }
}



