use assert_cmd::Command;

#[allow(dead_code)]
pub fn rew() -> Command {
    command("rew")
}

#[allow(dead_code)]
pub fn cpb() -> Command {
    command("cpb")
}

#[allow(dead_code)]
pub fn mvb() -> Command {
    command("mvb")
}

pub fn command(name: &str) -> Command {
    Command::cargo_bin(name).unwrap()
}
