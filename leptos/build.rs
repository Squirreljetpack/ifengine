use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const ICON_PATH: &str = "../assets/imgs/icon.png";

fn to_absolute(p: impl AsRef<Path>) -> PathBuf {
    Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(p.as_ref())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={ICON_PATH}");

    let icon_file = to_absolute(ICON_PATH);
    if let Ok(img) = image::open(&icon_file) {
        let public_dir = to_absolute("public");
        fs::create_dir_all(&public_dir).ok();

        let target_256 = public_dir.join("icon-256.png");
        let resized_256 = img.resize_exact(256, 256, image::imageops::FilterType::Lanczos3);
        let _ = resized_256.save(&target_256);

        let favicon_path = public_dir.join("favicon.ico");
        let resized_fav = img.resize_exact(64, 64, image::imageops::FilterType::Lanczos3);
        let _ = resized_fav.save(&favicon_path);
    }
}
