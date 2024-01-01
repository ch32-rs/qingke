use std::path::PathBuf;
use std::{env, fs};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Put the linker script somewhere the linker can find it.
    let has_highcode_feature = env::var("CARGO_FEATURE_HIGHCODE").is_ok();
    if has_highcode_feature {
        fs::write(out_dir.join("link.x"), include_bytes!("link-highcode.x")).unwrap();
    } else {
        fs::write(out_dir.join("link.x"), include_bytes!("link-no-highcode.x")).unwrap();
    }

    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=link-highcode.x");
    println!("cargo:rerun-if-changed=link-no-highcode.x");
    println!("cargo:rerun-if-changed=build.rs");
}
