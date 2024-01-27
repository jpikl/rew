#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn first() {
    let tc = Tc::cmd("last").stdin("a\nbc\n");
    tc.clone().ok("bc\n");
    tc.clone().arg("0").ok("");
    tc.clone().arg("1").ok("bc\n");
    tc.clone().arg("2").ok("a\nbc\n");
    tc.clone().arg("3").ok("a\nbc\n");

    // Same hash as `seq 1 20000 | tail -n10000 | md5sum`
    Tc::shell("seq 1 20000 | %bin% last 10000 | md5sum")
        .ok("8857ef28723cc4788a8ca7456214fc0c  -\n");
}
