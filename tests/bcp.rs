use assert_cmd::Command;

#[test]
fn no_args() {
    command().assert().success();
}

fn command() -> Command {
    Command::cargo_bin("bcp").unwrap()
}
