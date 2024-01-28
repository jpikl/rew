#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn split() {
    let tc = Tc::cmd("split").arg(":");

    tc.clone().stdin("").ok("\n");
    tc.clone().stdin("\n").ok("\n");
    tc.clone().stdin(":").ok("\n\n");
    tc.clone().stdin(":\n").ok("\n\n");
    tc.clone().stdin("::").ok("\n\n\n");
    tc.clone().stdin("::\n").ok("\n\n\n");
    tc.clone().stdin("a:b:c").ok("a\nb\nc\n");
    tc.clone().stdin("a:b:c\n").ok("a\nb\nc\n");
    tc.clone().stdin("a:b:c:").ok("a\nb\nc\n\n");
    tc.clone().stdin("a:b:c:\n").ok("a\nb\nc\n\n");

    let tc = tc.clone().arg("-t");

    tc.clone().stdin("").ok("\n");
    tc.clone().stdin("\n").ok("\n");
    tc.clone().stdin(":").ok("\n");
    tc.clone().stdin(":\n").ok("\n");
    tc.clone().stdin("::").ok("\n\n");
    tc.clone().stdin("::\n").ok("\n\n");
    tc.clone().stdin("a:b:c").ok("a\nb\nc\n");
    tc.clone().stdin("a:b:c\n").ok("a\nb\nc\n");
    tc.clone().stdin("a:b:c:").ok("a\nb\nc\n");
    tc.clone().stdin("a:b:c:\n").ok("a\nb\nc\n");
}
