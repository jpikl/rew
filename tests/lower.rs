#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn lower() {
    Tc::cmd("lower").stdin("aBčĎ").ok("abčď");
}
