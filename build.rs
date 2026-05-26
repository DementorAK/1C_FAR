use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let current_version = std::env::var("CARGO_PKG_VERSION").unwrap();
    let mut build_num_path = PathBuf::from(&manifest_dir);
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
    println!("cargo:rerun-if-changed=dist");

    // In release mode, assemble the dist package directory.
    // The library binary itself is copied by the CI/install workflow after linking,
    // because build.rs runs before compilation. Here we prepare everything else.
    let profile = std::env::var("PROFILE").unwrap_or_default();
    if profile == "release" {
        assemble_dist(&manifest_dir);
    }
}

/// Assembles the dist directory for the current feature target.
///
/// Output structure:
///   target/release/far3/   — for `--features far3` (default)
///     far1c_en.lng
///     far1c_ru.lng
///
///   target/release/far2/   — for `--features far2 --no-default-features`
///     far1c_en.lng
///     far1c_ru.lng
///     copy_to_far2l.sh
///
/// The library binary (far1c.dll / far1c.far-plug-wide) is copied into this
/// directory by the CI workflow or install script AFTER `cargo build` completes.
fn assemble_dist(manifest_dir: &str) {
    let is_far2 = std::env::var("CARGO_FEATURE_FAR2").is_ok();
    let feature_name = if is_far2 { "far2" } else { "far3" };

    // Resolve target/release/<feature>/ relative to the manifest (project root).
    // OUT_DIR is something like target/release/build/far1c-<hash>/out,
    // so we go up 4 levels to reach target/release/.
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let release_dir = PathBuf::from(&out_dir)
        .ancestors()
        .nth(4)
        .expect("Unexpected OUT_DIR depth")
        .to_path_buf();

    let dist_dir = release_dir.join(feature_name);
    fs::create_dir_all(&dist_dir).expect("Failed to create dist directory");

    // Copy language files from dist/ source
    let src_dist = PathBuf::from(manifest_dir).join("dist");
    for lng in &["far1c_en.lng", "far1c_ru.lng"] {
        let src = src_dist.join(lng);
        let dst = dist_dir.join(lng);
        fs::copy(&src, &dst)
            .unwrap_or_else(|e| panic!("Failed to copy {}: {}", src.display(), e));
    }

    // Copy install script for far2 builds
    if is_far2 {
        let src = src_dist.join("copy_to_far2l.sh");
        let dst = dist_dir.join("copy_to_far2l.sh");
        fs::copy(&src, &dst)
            .unwrap_or_else(|e| panic!("Failed to copy copy_to_far2l.sh: {}", e));
    }

    println!(
        "cargo:warning=Dist package prepared: {}",
        dist_dir.display()
    );
}
