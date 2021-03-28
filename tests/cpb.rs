#[path = "utils.rs"]
mod utils;

use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::prelude::*;
use utils::cpb;

#[test]
fn no_input() {
    cpb().assert().success();
}

mod input_terminator {
    use super::*;

    #[test]
    fn line() {
        let dir = TempDir::new().unwrap();
        let src_file = dir.child("a");
        src_file.write_str("1").unwrap();
        let dst_file = dir.child("b");

        cpb()
            .current_dir(dir.path())
            .write_stdin("<a\n>b")
            .assert()
            .success()
            .stdout("")
            .stderr("");

        src_file.assert("1");
        dst_file.assert("1");
    }

    #[test]
    fn null() {
        let dir = TempDir::new().unwrap();
        let src_file = dir.child("a");
        src_file.write_str("1").unwrap();
        let dst_file = dir.child("b");

        cpb()
            .current_dir(dir.path())
            .arg("--read-nul")
            .write_stdin("<a\0>b")
            .assert()
            .success()
            .stdout("")
            .stderr("");

        src_file.assert("1");
        dst_file.assert("1");
    }
}

mod failure {
    use super::*;

    #[test]
    fn immediate() {
        let dir = TempDir::new().unwrap();
        let src_file_1 = dir.child("a1");
        let src_file_2 = dir.child("a2");
        src_file_2.write_str("2").unwrap();
        let dst_file_1 = dir.child("b1");
        let dst_file_2 = dir.child("b2");

        cpb()
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
        let dir = TempDir::new().unwrap();
        let src_file_1 = dir.child("a1");
        let src_file_2 = dir.child("a2");
        src_file_2.write_str("2").unwrap();
        let dst_file_1 = dir.child("b1");
        let dst_file_2 = dir.child("b2");

        cpb()
            .current_dir(dir.path())
            .arg("--fail-at-end")
            .write_stdin("<a1\n>b1\n<a2\n>b2")
            .assert()
            .failure()
            .code(1)
            .stdout("")
            .stderr("error: Path 'a1' not found or user lacks permission\n");

        src_file_1.assert(predicates::path::missing());
        src_file_2.assert("2");
        dst_file_1.assert(predicates::path::missing());
        dst_file_2.assert("2");
    }
}

mod verbose {
    use super::*;

    #[test]
    fn success() {
        let dir = TempDir::new().unwrap();
        let src_file = dir.child("a");
        src_file.write_str("1").unwrap();
        let dst_file = dir.child("b");

        cpb()
            .current_dir(dir.path())
            .arg("--verbose")
            .write_stdin("<a\n>b")
            .assert()
            .success()
            .stdout("Copying 'a' to 'b' ... OK\n")
            .stderr("");

        src_file.assert(predicates::path::is_file());
        src_file.assert("1");
        dst_file.assert(predicates::path::is_file());
        dst_file.assert("1");
    }

    mod failure {
        use super::*;

        #[test]
        fn immediate() {
            let dir = TempDir::new().unwrap();
            let src_file_1 = dir.child("a1");
            let src_file_2 = dir.child("a2");
            src_file_2.write_str("2").unwrap();
            let dst_file_1 = dir.child("b1");
            let dst_file_2 = dir.child("b2");

            cpb()
                .current_dir(dir.path())
                .arg("--verbose")
                .write_stdin("<a1\n>b1\n<a2\n>b2")
                .assert()
                .failure()
                .code(1)
                .stdout("Copying 'a1' to 'b1' ... FAILED\n")
                .stderr("error: Path 'a1' not found or user lacks permission\n");

            src_file_1.assert(predicates::path::missing());
            src_file_2.assert("2");
            dst_file_1.assert(predicates::path::missing());
            dst_file_2.assert(predicates::path::missing());
        }

        #[test]
        fn at_end() {
            let dir = TempDir::new().unwrap();
            let src_file_1 = dir.child("a1");
            let src_file_2 = dir.child("a2");
            src_file_2.write_str("2").unwrap();
            let dst_file_1 = dir.child("b1");
            let dst_file_2 = dir.child("b2");

            cpb()
                .current_dir(dir.path())
                .arg("--verbose")
                .arg("--fail-at-end")
                .write_stdin("<a1\n>b1\n<a2\n>b2")
                .assert()
                .failure()
                .code(1)
                .stdout("Copying 'a1' to 'b1' ... FAILED\nCopying 'a2' to 'b2' ... OK\n")
                .stderr("error: Path 'a1' not found or user lacks permission\n");

            src_file_1.assert(predicates::path::missing());
            src_file_2.assert("2");
            dst_file_1.assert(predicates::path::missing());
            dst_file_2.assert("2");
        }
    }
}

#[test]
fn help() {
    cpb()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stderr("");
}
