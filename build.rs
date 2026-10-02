#[path = "build/cfgs.rs"]
mod cfgs;

fn main() {
    cfgs::emit_selected_leaf_cfgs();
}
