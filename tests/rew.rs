use assert_cmd::Command;

#[test]
fn no_args() {
    cmd().assert().failure();
}

#[test]
fn no_paths() {
    cmd().arg("_{p}_").assert().success().stdout("").stderr("");
}

#[test]
fn args_paths() {
    cmd()
        .arg("_{p}_")
        .arg("abc")
        .arg("def")
        .assert()
        .success()
        .stdout("_abc_\n_def_\n")
        .stderr("");
}

#[test]
fn stdin_paths() {
    cmd()
        .arg("_{p}_")
        .write_stdin("abc\ndef")
        .assert()
        .success()
        .stdout("_abc_\n_def_\n")
        .stderr("");
}

fn cmd() -> Command {
    Command::cargo_bin("rew").unwrap()
}
