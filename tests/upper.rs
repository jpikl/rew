#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn upper() {
    Tc::cmd("upper").stdin("aBčĎ").ok("ABČĎ");
}
