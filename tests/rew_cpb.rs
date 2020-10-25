#[path = "utils.rs"]
mod utils;

use indoc::indoc;
use utils::{cpb, rew};

#[test]
fn test() {
    let rew = rew()
        .arg("--batch")
        .arg("{f}.bak")
        .write_stdin("abc\ndef")
        .output()
        .unwrap();

    cpb()
        .arg("--verbose")
        .write_stdin(rew.stdout)
        .assert()
        .stdout(indoc! {"
            Copying 'abc' to 'abc.bak' ... OK
            Copying 'def' to 'def.bak' ... OK
        "})
        .stderr("");
}
