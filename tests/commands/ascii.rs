use crate::utils::Tc;

#[test]
fn test() {
    let tc = Tc::cmd("ascii");
    tc.clone().stdin("abCD").ok("abCD");
    tc.clone().stdin("ábČD").ok("abCD");
    tc.clone().arg("-d").stdin("ábČD").ok("bD");
}
