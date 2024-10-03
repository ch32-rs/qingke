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
