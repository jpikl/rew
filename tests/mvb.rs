#[path = "utils.rs"]
mod utils;

use assert_fs::prelude::*;
use predicates::prelude::*;
use utils::{mvb, temp_dir, write};

#[test]
fn no_input() {
    mvb().assert().success();
}

mod input_terminator {
    use super::*;

    #[test]
    fn line() {
        let dir = temp_dir();

        let src_file = write(dir.child("a"), "1");
        let dst_file = dir.child("b");

        mvb()
            .current_dir(dir.path())
            .write_stdin("<a\n>b")
            .assert()
            .success()
            .stdout("")
            .stderr("");

        src_file.assert(predicates::path::missing());
        dst_file.assert("1");
    }

    #[test]
    fn null() {
        let dir = temp_dir();

        let src_file = write(dir.child("a"), "1");
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
        dst_file.assert("1");
    }
}

mod failure {
    use super::*;

    #[test]
    fn immediate() {
        let dir = temp_dir();

        let src_file_1 = dir.child("a1");
        let src_file_2 = write(dir.child("a2"), "2");

        let dst_file_1 = dir.child("b1");
        let dst_file_2 = dir.child("b2");

        mvb()
            .current_dir(dir.path())
            .write_stdin("<a1\n>b1\n<a2\n>b2")
            .assert()
            .failure()
            .code(1)
            .stdout("")
            .stderr("error: Path 'a1' not found or user lacks permission\n");

        src_file_1.assert(predicates::path::missing());
        src_file_2.assert("2");

        dst_file_1.assert(predicates::path::missing());
        dst_file_2.assert(predicates::path::missing());
    }

    #[test]
    fn at_end() {
        let dir = temp_dir();

        let src_file_1 = dir.child("a1");
        let src_file_2 = write(dir.child("a2"), "2");

        let dst_file_1 = dir.child("b1");
        let dst_file_2 = dir.child("b2");

        mvb()
            .current_dir(dir.path())
            .arg("--fail-at-end")
            .write_stdin("<a1\n>b1\n<a2\n>b2")
            .assert()
            .failure()
            .code(1)
            .stdout("")
            .stderr("error: Path 'a1' not found or user lacks permission\n");

        src_file_1.assert(predicates::path::missing());
        src_file_2.assert(predicates::path::missing());

        dst_file_1.assert(predicates::path::missing());
        dst_file_2.assert("2");
    }
}

mod verbose {
    use super::*;

    #[test]
    fn success() {
        let dir = temp_dir();

        let src_file = write(dir.child("a"), "1");
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
        dst_file.assert("1");
    }

    mod failure {
        use super::*;

        #[test]
        fn immediate() {
            let dir = temp_dir();

            let src_file_1 = dir.child("a1");
            let src_file_2 = write(dir.child("a2"), "2");

            let dst_file_1 = dir.child("b1");
            let dst_file_2 = dir.child("b2");

            mvb()
                .current_dir(dir.path())
                .arg("--verbose")
                .write_stdin("<a1\n>b1\n<a2\n>b2")
                .assert()
                .failure()
                .code(1)
                .stdout("Moving 'a1' to 'b1' ... FAILED\n")
                .stderr("error: Path 'a1' not found or user lacks permission\n");

            src_file_1.assert(predicates::path::missing());
            src_file_2.assert("2");

            dst_file_1.assert(predicates::path::missing());
            dst_file_2.assert(predicates::path::missing());
        }

        #[test]
        fn at_end() {
            let dir = temp_dir();

            let src_file_1 = dir.child("a1");
            let src_file_2 = write(dir.child("a2"), "2");

            let dst_file_1 = dir.child("b1");
            let dst_file_2 = dir.child("b2");

            mvb()
                .current_dir(dir.path())
                .arg("--verbose")
                .arg("--fail-at-end")
                .write_stdin("<a1\n>b1\n<a2\n>b2")
                .assert()
                .failure()
                .code(1)
                .stdout("Moving 'a1' to 'b1' ... FAILED\nMoving 'a2' to 'b2' ... OK\n")
                .stderr("error: Path 'a1' not found or user lacks permission\n");

            src_file_1.assert(predicates::path::missing());
            src_file_2.assert(predicates::path::missing());

            dst_file_1.assert(predicates::path::missing());
            dst_file_2.assert("2");
        }
    }
}

#[test]
fn help() {
    mvb()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stderr("");
}
