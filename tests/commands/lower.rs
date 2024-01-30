use crate::utils::Tc;

#[test]
fn test() {
    let tc = Tc::cmd("lower");
    tc.clone().stdin("abCD").ok("abcd");
    tc.clone().stdin("ábČD").ok("ábčd");
}
