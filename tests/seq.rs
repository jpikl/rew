#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn seq() {
    let tc = Tc::cmd("seq");

    tc.clone().arg("1..10").ok("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n");
    tc.clone().arg("10..1").ok("10\n9\n8\n7\n6\n5\n4\n3\n2\n1\n");

    tc.clone().arg("-10..-1").ok("-10\n-9\n-8\n-7\n-6\n-5\n-4\n-3\n-2\n-1\n");
    tc.clone().arg("-1..-10").ok("-1\n-2\n-3\n-4\n-5\n-6\n-7\n-8\n-9\n-10\n");

    tc.clone().arg("-1..1").ok("-1\n0\n1\n");
    tc.clone().arg("1..-1").ok("1\n0\n-1\n");

    tc.clone().arg("2..9").arg("-s2").ok("2\n4\n6\n8\n");
    tc.clone().arg("2..10").arg("-s2").ok("2\n4\n6\n8\n10\n");
    tc.clone().arg("9..2").arg("-s-3").ok("9\n6\n3\n");
    tc.clone().arg("9..0").arg("-s-3").ok("9\n6\n3\n0\n");

    tc.clone().arg("-1..-1").ok("-1\n");
    tc.clone().arg("0..0").ok("0\n");
    tc.clone().arg("1..1").ok("1\n");

    Tc::shell("%bin% seq | head -n10").ok("1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n");
    Tc::shell("%bin% seq 10.. | head -n10").ok("10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n");
    Tc::shell("%bin% seq 10.. -s2 | head -n10").ok("10\n12\n14\n16\n18\n20\n22\n24\n26\n28\n");
    Tc::shell("%bin% seq 10.. -s-3 | head -n10").ok("10\n7\n4\n1\n-2\n-5\n-8\n-11\n-14\n-17\n");

    tc.clone()
        .arg(format!("{}..", i128::MAX))
        .err("error: number sequence overflown interger limit\n");
}
