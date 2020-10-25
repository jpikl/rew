#[path = "utils.rs"]
mod utils;

use indoc::indoc;
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
        .stdout("")
        .stderr("");
}

#[test]
fn nul_input_separator() {
    cpb()
        .arg("--read-nul")
        .write_stdin("<abc\0>def")
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

#[test]
fn verbose_output() {
    cpb()
        .arg("--verbose")
        .write_stdin(indoc! {"
            <abc
            >def
            <ghi
            >jkl
        "})
        .assert()
        .success()
        .stdout(indoc! {"
            Copying 'abc' to 'def' ... OK
            Copying 'ghi' to 'jkl' ... OK
        "})
        .stderr("");
}
