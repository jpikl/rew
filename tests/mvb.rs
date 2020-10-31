#[path = "utils.rs"]
mod utils;

use assert_fs::prelude::*;
use assert_fs::TempDir;
use utils::mvb;

#[test]
fn no_args() {
    mvb().assert().success();
}

#[test]
fn line_input_separator() {
    let dir = TempDir::new().unwrap();
    let src_file = dir.child("a");
    src_file.write_str("1").unwrap();
    let dst_file = dir.child("b");

    mvb()
        .current_dir(dir.path())
        .write_stdin("<a\n>b")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    src_file.assert(predicates::path::missing());
    dst_file.assert(predicates::path::is_file());
    dst_file.assert("1");
}

#[test]
fn nul_input_separator() {
    let dir = TempDir::new().unwrap();
    let src_file = dir.child("a");
    src_file.write_str("1").unwrap();
    let dst_file = dir.child("b");

    mvb()
        .current_dir(dir.path())
        .arg("--read-nul")
        .write_stdin("<a\0>b")
        .assert()
        .success()
        .stdout("")
        .stderr("");

    src_file.assert(predicates::path::missing());
    dst_file.assert(predicates::path::is_file());
    dst_file.assert("1");
}

#[test]
fn verbose_output_success() {
    let dir = TempDir::new().unwrap();
    let src_file = dir.child("a");
    src_file.write_str("1").unwrap();
    let dst_file = dir.child("b");

    mvb()
        .current_dir(dir.path())
        .arg("--verbose")
        .write_stdin("<a\n>b")
        .assert()
        .success()
        .stdout("Moving 'a' to 'b' ... OK\n")
        .stderr("");

    src_file.assert(predicates::path::missing());
    dst_file.assert(predicates::path::is_file());
    dst_file.assert("1");
}

#[test]
fn verbose_output_error() {
    let dir = TempDir::new().unwrap();
    let src_file = dir.child("a");
    let dst_file = dir.child("b");

    mvb()
        .current_dir(dir.path())
        .arg("--verbose")
        .write_stdin("<a\n>b")
        .assert()
        .failure()
        .code(1)
        .stdout("Moving 'a' to 'b' ... FAILED\n")
        .stderr("error: Path 'a' not found or user lacks permission\n");

    src_file.assert(predicates::path::missing());
    dst_file.assert(predicates::path::missing());
}
