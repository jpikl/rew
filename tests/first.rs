#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn first() {
    let tc = Tc::cmd("first").stdin("a\nbc\n");
    tc.clone().ok("a\n");
    tc.clone().arg("0").ok("");
    tc.clone().arg("1").ok("a\n");
    tc.clone().arg("2").ok("a\nbc\n");
    tc.clone().arg("3").ok("a\nbc\n");

    Tc::shell("seq 1 10000 | %bin% first 9999 | md5sum")
        .ok("05fda6bec6aabc94d0fc54380ace8412  -\n");
}
