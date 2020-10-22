#[path = "utils.rs"]
mod utils;

use utils::cpb;

#[test]
fn no_args() {
    cpb().assert().success();
}

#[test]
fn line_input_separator() {
    cpb()
        .write_stdin("<abc\n>def")
        .assert()
        .success()
        .stdout("Copying 'abc' to 'def'\n")
        .stderr("");
}

#[test]
fn nul_input_separator() {
    cpb()
        .arg("--read-nul")
        .write_stdin("<abc\0>def")
        .assert()
        .success()
        .stdout("Copying 'abc' to 'def'\n")
        .stderr("");
}
