use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let current_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let mut build_num_path = PathBuf::from(manifest_dir);
    build_num_path.push("build_num.txt");

    let mut build_num: u32 = 0;
    let mut last_version = String::new();

    if build_num_path.exists() {
        let content = fs::read_to_string(&build_num_path).unwrap_or_else(|_| "".to_string());
        if let Some((v, n)) = content.trim().split_once(':') {
            last_version = v.to_string();
            build_num = n.parse().unwrap_or(0);
        } else {
            // Backward compatibility: if only number is present
            build_num = content.trim().parse().unwrap_or(0);
        }
    }

    if current_version != last_version {
        build_num = 1;
    } else {
        build_num += 1;
    }

    fs::write(
        &build_num_path,
        format!("{}:{}", current_version, build_num),
    )
    .expect("Failed to write build_num.txt");

    let mut version_rs_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    version_rs_path.push("version.rs");

    let version_content = format!(
        "#[allow(dead_code)]\npub const VERSION: &str = \"{}\";\npub const BUILD_NUMBER: u32 = {};\n",
        current_version, build_num
    );
    // Write to OUT_DIR instead of src/ to avoid git noise and circular dependencies
    fs::write(&version_rs_path, version_content).expect("Failed to write version.rs");

    // Export the command prefix
    println!("cargo:rustc-env=PLUGIN_PREFIX=1c");

    // Ensure we rerun if any source changes or version changes
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=Cargo.toml");
    println!("cargo:rerun-if-changed=build_num.txt");
}
