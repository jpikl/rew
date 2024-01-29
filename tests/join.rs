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

    let buf = tc.clone().arg("--buf-size=8");

    buf.clone().stdin("aaaaaaa\nb").ok("aaaaaaa:b\n");
    buf.clone().stdin("aaaaaaaa\nb").ok("aaaaaaaa:b\n");
    buf.clone().stdin("aaaaaaaaa\nb").ok("aaaaaaaaa:b\n");

    let tc = tc.clone().arg("-t");

    tc.clone().stdin("").ok(":\n");
    tc.clone().stdin("\n").ok(":\n");
    tc.clone().stdin("\n\n").ok("::\n");
    tc.clone().stdin("a\nb\nc").ok("a:b:c:\n");
    tc.clone().stdin("a\nb\nc\n").ok("a:b:c:\n");

    let buf = tc.clone().arg("--buf-size=8");

    buf.clone().stdin("aaaaaaa\nb").ok("aaaaaaa:b:\n");
    buf.clone().stdin("aaaaaaaa\nb").ok("aaaaaaaa:b:\n");
    buf.clone().stdin("aaaaaaaaa\nb").ok("aaaaaaaaa:b:\n");
}
