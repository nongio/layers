extern crate cbindgen;

use cbindgen::Config;
use std::env;
use std::path::PathBuf;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let package_name = env::var("CARGO_PKG_NAME").unwrap();
    let output_file = target_dir()
        .join(format!("{}.h", package_name))
        .display()
        .to_string();

    let mut config = Config::from_root_or_default("./");
    // Disable parsing dependencies to avoid issues with edition2024
    config.parse.parse_deps = false;
    config.parse.expand.default_features = false;

    println!("crate_dir: {}", crate_dir);
    println!("output_file: {}", output_file);
    println!("config: {:?}", config);

    match cbindgen::generate_with_config(&crate_dir, config) {
        Ok(bindings) => {
            bindings.write_to_file(&output_file);
        }
        Err(e) => {
            eprintln!("Warning: cbindgen failed to generate bindings: {}", e);
            eprintln!("Continuing build without C bindings...");
        }
    }
}

/// Find the location of the `target/` directory. Note that this may be
/// overridden by `cmake`, so we also need to check the `CARGO_TARGET_DIR`
/// variable.
fn target_dir() -> PathBuf {
    if let Ok(target) = env::var("CARGO_TARGET_DIR") {
        PathBuf::from(target)
    } else {
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("target")
    }
}
