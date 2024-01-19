#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn upper() {
    let tc = Tc::cmd("upper");
    tc.clone().stdin("abCD").ok("ABCD");
    tc.clone().stdin("ábČD").ok("ÁBČD");
}
