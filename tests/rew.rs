#[path = "utils.rs"]
mod utils;

use indoc::indoc;
use predicates::str::starts_with;
use std::path::Path;
use utils::rew;

#[test]
fn no_pattern_no_paths() {
    rew().assert().success().stdout("").stderr("");
}

#[test]
fn no_pattern_some_paths() {
    rew()
        .write_stdin("a\nb")
        .assert()
        .success()
        .stdout("a\nb\n")
        .stderr("");
}

#[test]
fn some_pattern_no_paths() {
    rew().arg("_").assert().success().stdout("").stderr("");
}

#[test]
fn paths_from_args() {
    rew()
        .arg("_{}_")
        .arg("a")
        .arg("b")
        .assert()
        .success()
        .stdout("_a_\n_b_\n")
        .stderr("");
}

#[test]
fn paths_from_args_over_stdin() {
    rew()
        .arg("_{}_")
        .arg("a")
        .arg("b")
        .write_stdin("c")
        .assert()
        .success()
        .stdout("_a_\n_b_\n")
        .stderr("");
}

#[test]
fn paths_from_stdin() {
    rew()
        .arg("_{}_")
        .write_stdin("a\n\0b")
        .assert()
        .success()
        .stdout("_a_\n_\0b_\n")
        .stderr("");
}

#[test]
fn nul_input_delimiter() {
    rew()
        .arg("--read-nul")
        .arg("_{}_")
        .write_stdin("a\n\0b")
        .assert()
        .success()
        .stdout("_a\n_\n_b_\n")
        .stderr("");
}

#[test]
fn nul_output_delimiter() {
    rew()
        .arg("--print-nul")
        .arg("_{}_")
        .write_stdin("a\n\0b")
        .assert()
        .success()
        .stdout("_a_\0_\0b_\0")
        .stderr("");
}

#[test]
fn no_input_delimiter() {
    rew()
        .arg("--read-raw")
        .arg("_{}_")
        .write_stdin("a\n\0b")
        .assert()
        .success()
        .stdout("_a\n\0b_\n")
        .stderr("");
}

#[test]
fn no_output_delimiter() {
    rew()
        .arg("--print-raw")
        .arg("_{}_")
        .write_stdin("a\n\0b")
        .assert()
        .success()
        .stdout("_a__\0b_")
        .stderr("");
}

#[test]
fn custom_input_delimiter() {
    rew()
        .arg("--read=;")
        .arg("_{}_")
        .write_stdin("a;b")
        .assert()
        .success()
        .stdout("_a_\n_b_\n")
        .stderr("");
}

#[test]
fn custom_output_delimiter() {
    rew()
        .arg("--print=;")
        .arg("_{}_")
        .write_stdin("a\nb")
        .assert()
        .success()
        .stdout("_a_;_b_;")
        .stderr("");
}

#[test]
fn custom_no_trailing_output_delimiter() {
    rew()
        .arg("--print=;")
        .arg("--no-trailing-delimiter")
        .arg("_{}_")
        .write_stdin("a\nb")
        .assert()
        .success()
        .stdout("_a_;_b_")
        .stderr("");
}

#[test]
fn diff_output_mode() {
    rew()
        .arg("--diff")
        .arg("_{}_")
        .write_stdin("a\n\0b")
        .assert()
        .success()
        .stdout(indoc! {"
            <a
            >_a_
            <\0b
            >_\0b_
        "})
        .stderr("");
}

#[test]
fn pretty_output_mode() {
    rew()
        .arg("--pretty")
        .arg("_{}_")
        .write_stdin("a\n\0b")
        .assert()
        .success()
        .stdout(indoc! {"
            a -> _a_
            \0b -> _\0b_
        "})
        .stderr("");
}

#[test]
fn local_counter() {
    rew()
        .arg("--local-counter=2:3")
        .arg("{}.{c}.{C}")
        .write_stdin(indoc! {"
            a/a
            a/b
            b/a
            b/b
        "})
        .assert()
        .success()
        .stdout(indoc! {"
            a/a.2.1
            a/b.5.2
            b/a.2.3
            b/b.5.4
        "})
        .stderr("");
}

#[test]
fn global_counter() {
    rew()
        .arg("--global-counter=2:3")
        .arg("{}.{c}.{C}")
        .write_stdin(indoc! {"
            a/a
            a/b
            b/a
            b/b
        "})
        .assert()
        .success()
        .stdout(indoc! {"
            a/a.1.2
            a/b.2.5
            b/a.1.8
            b/b.2.11
        "})
        .stderr("");
}

#[test]
fn pattern_parse_error() {
    rew()
        .arg("{")
        .assert()
        .failure()
        .code(3)
        .stdout("")
        .stderr(indoc! {"
            error: Invalid pattern: No matching '}' after expression start
            
            {
            ^
        "});
}

#[test]
fn pattern_eval_error() {
    rew()
        .arg("{A}")
        .write_stdin(indoc! {"
            non-existent
            Cargo.toml
        "})
        .assert()
        .failure()
        .code(4)
        .stdout("")
        .stderr(starts_with(
            "error: 'Canonical path' evaluation failed for value 'non-existent':",
        ));
}

#[test]
fn pattern_eval_error_at_end() {
    rew()
        .arg("--fail-at-end")
        .arg("{A}")
        .write_stdin(indoc! {"
            non-existent
            Cargo.toml
        "})
        .assert()
        .failure()
        .code(4)
        .stdout(format!(
            "{}\n",
            Path::new("Cargo.toml")
                .canonicalize()
                .unwrap()
                .to_string_lossy()
        ))
        .stderr(starts_with(
            "error: 'Canonical path' evaluation failed for value 'non-existent':",
        ));
}

#[test]
fn explanation() {
    rew()
        .arg("--explain")
        .arg("_")
        .assert()
        .success()
        .stdout(indoc! {"
            _
            ^
            
            Constant '_'
            
        "})
        .stderr("");
}
