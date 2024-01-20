#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn stream() {
    let tc = Tc::cmd("stream");

    tc.clone().ok("");
    tc.clone().arg("a").ok("a\n");
    tc.clone().arg("a").arg("bc").ok("a\nbc\n");
    tc.clone().arg("a").arg("bc").arg("def").ok("a\nbc\ndef\n");
}
