#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn ascii() {
    let tc = Tc::cmd("ascii").stdin("áB\nčĎ\neF\n");
    tc.clone().ok("aB\ncD\neF\n");
    tc.clone().arg("-d").ok("B\n\neF\n");
}
