use crate::utils::Tc;

#[test]
fn test() {
    let tc = Tc::cmd("skip").stdin("a\nbc\n");
    tc.clone().arg("0").ok("a\nbc\n");
    tc.clone().arg("1").ok("bc\n");
    tc.clone().arg("2").ok("");
    tc.clone().arg("3").ok("");

    // Same hash as `seq 1 20000 | tail -n-10000 | md5sum`
    Tc::shell("seq 1 20000 | %bin% skip 10000 | md5sum")
        .ok("8857ef28723cc4788a8ca7456214fc0c  -\n");
}
