// Leaf → `rustc-cfg` mapping for the `qingke` package only (`build.rs` uses `mod cfgs`).
// Leaf names match `[features]` in `qingke/Cargo.toml`; CSR caps use the `csr_*` prefix.

use std::collections::HashSet;
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
        caps: &["dm_dataaddr_0f4", "cs_mstatus", "csr_corecfgr", "csr_intsyscr", "csr_mtvec"],
    },
    LeafSpec {
        leaf: "v2c",
        family: "qingke_v2",
        caps: &["dm_dataaddr_0f4", "cs_mstatus", "csr_corecfgr", "csr_intsyscr", "csr_mtvec"],
    },
    LeafSpec {
        leaf: "v3a",
        family: "qingke_v3",
        caps: &[
            "dm_dataaddr_380",
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
            "dm_dataaddr_380",
            "csr_inestcr",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    // No `dm_dataaddr_*`: neither the CH570 nor the CH585 EVT has an SDI
    // printf, so the debug module data address is unknown and `qingke::dm`
    // is left out rather than guessed.
    LeafSpec {
        leaf: "v3c",
        family: "qingke_v3",
        caps: &[
            "csr_inestcr",
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
            "dm_dataaddr_340",
            "csr_inestcr",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ],
    },
    // CH32V407/CH32V467: WCH EVT's SRC/Debug/debug.c and SDI_Printf
    // example use DATA0 = 0xE0000340 and DATA1 = 0xE0000344.
    LeafSpec {
        leaf: "v3v",
        family: "qingke_v3",
        caps: &[
            "dm_dataaddr_340",
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
            "dm_dataaddr_380",
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
            "dm_dataaddr_380",
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
            "dm_dataaddr_380",
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
            "dm_dataaddr_380",
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
            "dm_dataaddr_380",
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
            "dm_dataaddr_340",
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

fn sorted_unique<'a>(values: impl IntoIterator<Item = &'a str>) -> Vec<&'a str> {
    let set: HashSet<&'a str> = values.into_iter().collect();
    let mut out: Vec<&'a str> = set.into_iter().collect();
    out.sort_unstable();
    out
}

pub fn declare_check_cfgs() {
    let families = sorted_unique(LEAVES.iter().map(|spec| spec.family));
    let caps = sorted_unique(LEAVES.iter().flat_map(|spec| spec.caps.iter().copied()));

    for family in families {
        println!("cargo:rustc-check-cfg=cfg({})", family);
    }
    for cap in caps {
        println!("cargo:rustc-check-cfg=cfg({})", cap);
    }
}

pub fn emit_selected_leaf_cfgs() {
    declare_check_cfgs();

    let mut selected: Vec<&LeafSpec> = Vec::new();
    for spec in LEAVES {
        let key = format!("CARGO_FEATURE_{}", spec.leaf.to_ascii_uppercase());
        if env::var_os(&key).is_some() {
            selected.push(spec);
        }
    }

    match selected.len() {
        0 => {
            let pkg = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "qingke".into());
            panic!(
                "{pkg}: enable exactly one leaf core feature on this package \
                 (v2a, v3f, v5f, …); see Cargo.toml [features]"
            );
        }
        1 => {
            let spec = selected[0];
            println!("cargo:rustc-cfg={}", spec.family);
            for cap in spec.caps {
                println!("cargo:rustc-cfg={}", cap);
            }
            println!("cargo:rerun-if-changed=build/cfgs.rs");
        }
        _ => {
            let names: Vec<_> = selected.iter().map(|s| s.leaf).collect();
            let pkg = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "qingke".into());
            panic!(
                "{pkg}: at most one leaf core feature may be enabled, got: {}",
                names.join(", ")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LEAVES;

    #[test]
    fn v3b_capabilities_support_evt_interrupt_setup() {
        let spec = LEAVES.iter().find(|spec| spec.leaf == "v3b").unwrap();
        assert_eq!(spec.family, "qingke_v3");
        for cap in [
            "csr_inestcr",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ] {
            assert!(spec.caps.contains(&cap));
        }
        assert!(!spec.caps.contains(&"pfic_v3"));
        assert!(!spec.caps.contains(&"cs_mstatus"));
    }

    #[test]
    fn v3v_capabilities_match_evt_without_old_pfic_layout() {
        let spec = LEAVES.iter().find(|spec| spec.leaf == "v3v").unwrap();
        assert_eq!(spec.family, "qingke_v3");
        for cap in [
            "csr_inestcr",
            "csr_gintenr",
            "csr_corecfgr",
            "csr_intsyscr",
            "csr_mtvec",
        ] {
            assert!(spec.caps.contains(&cap));
        }
        let dm_caps: Vec<_> = spec
            .caps
            .iter()
            .copied()
            .filter(|cap| cap.starts_with("dm_dataaddr_"))
            .collect();
        assert_eq!(dm_caps, ["dm_dataaddr_340"]);
        assert!(!spec.caps.contains(&"pfic_v3"));
        assert!(!spec.caps.contains(&"cs_mstatus"));
    }
}
