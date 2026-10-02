use std::collections::HashSet;
use std::path::PathBuf;
use std::{env, fs};

/// Parse the target RISC-V architecture and returns its bit width and the extension set
fn parse_target(target: &str, cargo_flags: &str) -> (u32, HashSet<char>) {
    // isolate bit width and extensions from the rest of the target information
    let arch = target
        .trim_start_matches("riscv")
        .split('-')
        .next()
        .unwrap();

    let bits = arch
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse::<u32>()
        .unwrap();

    let mut extensions: HashSet<char> = arch.chars().skip_while(|c| c.is_ascii_digit()).collect();
    // expand the 'g' shorthand extension
    if extensions.contains(&'g') {
        extensions.insert('i');
        extensions.insert('m');
        extensions.insert('a');
        extensions.insert('f');
        extensions.insert('d');
    }

    let cargo_flags = cargo_flags
        .split(0x1fu8 as char)
        .filter(|arg| !arg.is_empty());

    cargo_flags
        .filter(|k| k.starts_with("target-feature="))
        .flat_map(|str| {
            let flags = str.split('=').collect::<Vec<&str>>()[1];
            flags.split(',')
        })
        .for_each(|feature| {
            let chars = feature.chars().collect::<Vec<char>>();
            match chars[0] {
                '+' => {
                    extensions.insert(chars[1]);
                }
                '-' => {
                    extensions.remove(&chars[1]);
                }
                _ => {
                    panic!("Unsupported target feature operation");
                }
            }
        });

    (bits, extensions)
}

/// Leaf features are the `[features]` entries that forward to `qingke/…` (see `Cargo.toml`).
fn leaf_feature_names_from_manifest() -> Vec<String> {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
    let contents = fs::read_to_string(&manifest).unwrap();
    let mut in_features = false;
    let mut leaves = Vec::new();
    for line in contents.lines() {
        let line = line.split('#').next().unwrap().trim();
        if line == "[features]" {
            in_features = true;
            continue;
        }
        if !in_features {
            continue;
        }
        if line.starts_with('[') {
            break;
        }
        if let Some((name, value)) = line.split_once('=') {
            let name = name.trim();
            if value.contains("qingke/") {
                leaves.push(name.to_string());
            }
        }
    }
    leaves
}

fn leaf_feature_enabled(leaf: &str) -> bool {
    let key = format!("CARGO_FEATURE_{}", leaf.to_ascii_uppercase());
    env::var_os(&key).is_some()
}

fn assert_one_leaf_feature() {
    let leaves = leaf_feature_names_from_manifest();
    let selected: Vec<&str> = leaves
        .iter()
        .map(String::as_str)
        .filter(|leaf| leaf_feature_enabled(leaf))
        .collect();
    match selected.len() {
        0 => panic!(
            "qingke-rt: enable exactly one leaf core feature (v2a, v3f, v5f, …); see Cargo.toml [features]"
        ),
        1 => {}
        _ => panic!(
            "qingke-rt: at most one leaf core feature may be enabled, got: {}",
            selected.join(", ")
        ),
    }
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Put the linker script somewhere the linker can find it.
    let has_highcode_feature = env::var("CARGO_FEATURE_HIGHCODE").is_ok();
    if has_highcode_feature {
        fs::write(out_dir.join("link.x"), include_bytes!("link-highcode.x")).unwrap();
    } else {
        fs::write(out_dir.join("link.x"), include_bytes!("link-no-highcode.x")).unwrap();
    }

    assert_one_leaf_feature();

    // QingKe V2 requires the vector table to be 1KB-aligned
    let has_v2 = leaf_feature_enabled("v2a") || leaf_feature_enabled("v2c");
    let asserts: &[u8] = match (has_v2, has_highcode_feature) {
        (true, true) => include_bytes!("assert-v2-align-highcode.x"),
        (true, false) => include_bytes!("assert-v2-align-no-highcode.x"),
        _ => b"",
    };
    fs::write(out_dir.join("assert-align.x"), asserts).unwrap();

    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=link-highcode.x");
    println!("cargo:rerun-if-changed=link-no-highcode.x");
    println!("cargo:rerun-if-changed=assert-v2-align-highcode.x");
    println!("cargo:rerun-if-changed=assert-v2-align-no-highcode.x");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let target = env::var("TARGET").unwrap();
    let cargo_flags = env::var("CARGO_ENCODED_RUSTFLAGS").unwrap();
    // set configuration flags depending on the target

    println!("cargo::rustc-check-cfg=cfg(riscvf)");
    println!("cargo::rustc-check-cfg=cfg(riscvd)");

    if target.starts_with("riscv") {
        println!("cargo:rustc-cfg=riscv");

        // This is required until target_arch & target_feature risc-v work is
        // stable and in-use (rust 1.75.0)
        let (_bits, extensions) = parse_target(&target, &cargo_flags);

        // expose the ISA extensions
        for ext in &extensions {
            println!("cargo:rustc-cfg=riscv{}", ext);
        }
    }
}
