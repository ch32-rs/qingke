fn main() {
    mod cfgs {
        include!("build/cfgs.inc.rs");
    }
    cfgs::emit_selected_leaf_cfgs();
}
