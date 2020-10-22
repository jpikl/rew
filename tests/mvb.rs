#[path = "utils.rs"]
mod utils;

use utils::mvb;

#[test]
fn no_args() {
    mvb().assert().success();
}

#[test]
fn line_input_separator() {
    mvb()
        .write_stdin("<abc\n>def")
        .assert()
        .success()
        .stdout("Moving 'abc' to 'def'\n")
        .stderr("");
}

#[test]
fn nul_input_separator() {
    mvb()
        .arg("--read-nul")
        .write_stdin("<abc\0>def")
        .assert()
        .success()
        .stdout("Moving 'abc' to 'def'\n")
        .stderr("");
}
