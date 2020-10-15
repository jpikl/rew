use assert_cmd::Command;

#[test]
fn no_args() {
    cmd().assert().success();
}

fn cmd() -> Command {
    Command::cargo_bin("bcp").unwrap()
}
