#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn ascii() {
    let tc = Tc::cmd("ascii");
    tc.clone().stdin("abCD").ok("abCD");
    tc.clone().stdin("ábČD").ok("abCD");
    tc.clone().arg("-d").stdin("ábČD").ok("bD");
}
