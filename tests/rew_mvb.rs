#[path = "utils.rs"]
mod utils;

use assert_fs::prelude::*;
use utils::{mvb, rew, temp_dir, write};

#[test]
fn test() {
    let dir = temp_dir();
    let src_file = write(dir.child("a"), "1");
    let dst_file = dir.child("b");

    let rew = rew()
        .arg("--diff")
        .arg("b")
        .write_stdin("a")
        .output()
        .unwrap();

    mvb()
        .current_dir(dir.path())
        .arg("--verbose")
        .write_stdin(rew.stdout)
        .assert()
        .stdout("Moving 'a' to 'b' ... OK\n")
        .stderr("");

    src_file.assert(predicates::path::missing());
    dst_file.assert("1");
}
