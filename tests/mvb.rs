#[path = "utils.rs"]
mod utils;

use indoc::indoc;
use utils::mvb;

#[test]
fn no_args() {
    mvb().assert().success();
}

//TODO #[test]
fn line_input_separator() {
    mvb()
        .write_stdin("<abc\n>def")
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

//TODO #[test]
fn nul_input_separator() {
    mvb()
        .arg("--read-nul")
        .write_stdin("<abc\0>def")
        .assert()
        .success()
        .stdout("")
        .stderr("");
}

//TODO #[test]
fn verbose_output() {
    mvb()
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
            Moving 'abc' to 'def' ... OK
            Moving 'ghi' to 'jkl' ... OK
        "})
        .stderr("");
}
