#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn split() {
    let tc = Tc::cmd("join").arg(":");

    tc.clone().stdin("").ok("\n");
    tc.clone().stdin("\n").ok("\n");
    tc.clone().stdin("\n\n").ok(":\n");
    tc.clone().stdin("a\nb\nc").ok("a:b:c\n");
    tc.clone().stdin("a\nb\nc\n").ok("a:b:c\n");

    let tc = tc.clone().arg("-t");

    tc.clone().stdin("").ok(":\n");
    tc.clone().stdin("\n").ok(":\n");
    tc.clone().stdin("\n\n").ok("::\n");
    tc.clone().stdin("a\nb\nc").ok("a:b:c:\n");
    tc.clone().stdin("a\nb\nc\n").ok("a:b:c:\n");
}
