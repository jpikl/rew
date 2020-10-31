#[path = "utils.rs"]
mod utils;

use assert_fs::prelude::*;
use assert_fs::TempDir;
use utils::{cpb, rew};

#[test]
fn test() {
    let dir = TempDir::new().unwrap();
    let src_file = dir.child("a");
    src_file.write_str("1").unwrap();
    let dst_file = dir.child("b");

    let rew = rew()
        .arg("--bulk")
        .arg("b")
        .write_stdin("a")
        .output()
        .unwrap();

    cpb()
        .current_dir(dir.path())
        .arg("--verbose")
        .write_stdin(rew.stdout)
        .assert()
        .stdout("Copying 'a' to 'b' ... OK\n")
        .stderr("");

    src_file.assert(predicates::path::is_file());
    src_file.assert("1");
    dst_file.assert(predicates::path::is_file());
    dst_file.assert("1");
}
