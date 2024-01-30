use crate::utils::Tc;

#[test]
fn test() {
    let tc = Tc::cmd("upper");
    tc.clone().stdin("abCD").ok("ABCD");
    tc.clone().stdin("ábČD").ok("ÁBČD");
}
