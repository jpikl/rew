#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn lower() {
    let tc = Tc::cmd("lower");
    tc.clone().stdin("abCD").ok("abcd");
    tc.clone().stdin("ábČD").ok("ábčd");
}
