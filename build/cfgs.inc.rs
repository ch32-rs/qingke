// Shared by `qingke/build.rs` and `qingke-rt/build.rs`.
// Leaf core features (`v2a`, `v3f`, …) live in each crate's `Cargo.toml`.
// CSR availability uses the `csr_*` prefix (e.g. `csr_inestcr`, `csr_corecfgr`).

use std::env;

struct LeafSpec {
    leaf: &'static str,
    family: &'static str,
    caps: &'static [&'static str],
}

const LEAVES: &[LeafSpec] = &[
    LeafSpec {
        leaf: "v2a",
        family: "qingke_v2",
        caps: &["dm_data_f4", "cs_mstatus", "csr_corecfgr", "csr_intsyscr", "csr_mtvec"],
    },
    LeafSpec {
        leaf: "v2c",
        family: "qingke_v2",
        caps: &["dm_data_f4", "cs_mstatus", "csr_corecfgr", "csr_intsyscr", "csr_mtvec"],
    },
    LeafSpec {
        leaf: "v3a",
        family: "qingke_v3",
        caps: &[
            "dm_data_380",
            "pfic_v3",
            "cs_mstatus",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v3b",
        family: "qingke_v3",
        caps: &[
            "dm_data_380",
            "pfic_v3",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v3f",
        family: "qingke_v3",
        caps: &[
            "dm_data_340",
            "pfic_v3",
            "csr_inestcr",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v4a",
        family: "qingke_v4",
        caps: &[
            "dm_data_380",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v4b",
        family: "qingke_v4",
        caps: &[
            "dm_data_380",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v4c",
        family: "qingke_v4",
        caps: &[
            "dm_data_380",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v4f",
        family: "qingke_v4",
        caps: &[
            "dm_data_380",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v4j",
        family: "qingke_v4",
        caps: &[
            "dm_data_380",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    LeafSpec {
        leaf: "v5f",
        family: "qingke_v5",
        caps: &[
            "dm_data_340",
            "csr_inestcr",
            "csr_gintenr",
            "csr_cache_strtg_ctlr",
            "csr_cache_pmp_ovr",
            "csr_opcache_ctlr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
];

const FAMILIES: &[&str] = &["qingke_v2", "qingke_v3", "qingke_v4", "qingke_v5"];

const ALL_CAPS: &[&str] = &[
    "dm_data_f4",
    "dm_data_380",
    "dm_data_340",
    "pfic_v3",
    "cs_mstatus",
    "csr_corecfgr",
    "csr_intsyscr",
    "csr_mtvec",
    "csr_gintenr",
    "csr_inestcr",
    "csr_cache_strtg_ctlr",
    "csr_cache_pmp_ovr",
    "csr_opcache_ctlr",
];

pub fn declare_check_cfgs() {
    for family in FAMILIES {
        println!("cargo:rustc-check-cfg=cfg({})", family);
    }
    for cap in ALL_CAPS {
        println!("cargo:rustc-check-cfg=cfg({})", cap);
    }
}

pub fn emit_selected_leaf_cfgs() -> Option<&'static str> {
    declare_check_cfgs();

    let mut selected: Vec<&LeafSpec> = Vec::new();
    for spec in LEAVES {
        let key = format!("CARGO_FEATURE_{}", spec.leaf.to_ascii_uppercase());
        if env::var_os(&key).is_some() {
            selected.push(spec);
        }
    }

    match selected.len() {
        0 => None,
        1 => {
            let spec = selected[0];
            println!("cargo:rustc-cfg={}", spec.family);
            for cap in spec.caps {
                println!("cargo:rustc-cfg={}", cap);
            }
            println!("cargo:rerun-if-changed=build/cfgs.inc.rs");
            Some(spec.leaf)
        }
        _ => {
            let names: Vec<_> = selected.iter().map(|s| s.leaf).collect();
            panic!(
                "qingke: at most one leaf core feature may be enabled, got: {}",
                names.join(", ")
            );
        }
    }
}

pub fn leaf_enabled(leaf: &str) -> bool {
    let key = format!("CARGO_FEATURE_{}", leaf.to_ascii_uppercase());
    env::var_os(&key).is_some()
}
