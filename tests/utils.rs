use assert_cmd::assert::Assert;
use assert_cmd::Command;
use claims::assert_ok;

pub fn test_command(name: &str, args: &[&str], input: impl AsRef<[u8]>, output: impl AsRef<[u8]>) {
    build_command(name, args, input)
        .success()
        .stdout(output.as_ref().to_owned())
        .stderr("");
}

// This is actually used but rustc claims otherwise
#[allow(dead_code)]
pub fn test_command_failure(name: &str, args: &[&str], input: impl AsRef<[u8]>) {
    build_command(name, args, input).failure();
}

fn build_command(name: &str, args: &[&str], input: impl AsRef<[u8]>) -> Assert {
    let mut command = assert_ok!(Command::cargo_bin("rew"));
    command.arg(name);
    command.args(args);
    command.write_stdin(input.as_ref().to_owned());
    command.assert()
}
