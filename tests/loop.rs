#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn r#loop() {
    let tc = Tc::cmd("loop");

    let single = tc.clone().stdin("ab");
    single.clone().arg("0").ok("");
    single.clone().arg("1").ok("ab");
    single.clone().arg("2").ok("abab");
    single.clone().arg("3").ok("ababab");

    let multi = tc.clone().stdin("ab\nCD\n");
    multi.clone().arg("0").ok("");
    multi.clone().arg("1").ok("ab\nCD\n");
    multi.clone().arg("2").ok("ab\nCD\nab\nCD\n");
    multi.clone().arg("3").ok("ab\nCD\nab\nCD\nab\nCD\n");

    // Same hash as `for((i=0; i<3; i++)); do seq 1 10000; done | md5sum`
    Tc::shell("seq 1 10000 | %bin% loop 3 | md5sum").ok("5f0f7f173c062a0d128ff75ded51b09b  -\n");

    // Same hash as `for((i=0; i<10; i++)); do seq 1 10000; done | head -n40000 | md5sum`
    Tc::shell("seq 1 10000 | %bin% loop | head -n40000 | md5sum")
        .ok("3aeff2d35d8836dfbbdfe882848b5d30  -\n");
}
