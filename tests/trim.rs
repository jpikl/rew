#[path = "utils.rs"]
mod utils;

use utils::Tc;

#[test]
fn trim() {
    let tc = Tc::cmd("trim").stdin("  \n a \n  b  c  ");
    tc.clone().ok("\na\nb  c\n");
    tc.clone().arg("-s").ok("\na \nb  c  \n");
    tc.clone().arg("-e").ok("\n a\n  b  c\n");
    tc.clone().arg("-se").ok("\na\nb  c\n");
}
