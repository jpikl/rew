#[path = "utils.rs"]
mod utils;

use indoc::indoc;
use utils::{mvb, rew};

#[test]
fn test() {
    let rew = rew()
        .arg("--batch")
        .arg("{f}.bak")
        .write_stdin("abc\ndef")
        .output()
        .unwrap();

    mvb()
        .arg("--verbose")
        .write_stdin(rew.stdout)
        .assert()
        .stdout(indoc! {"
            Moving 'abc' to 'abc.bak' ... OK
            Moving 'def' to 'def.bak' ... OK
        "})
        .stderr("");
}
