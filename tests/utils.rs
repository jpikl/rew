use assert_cmd::Command;
use assert_fs::fixture::FileWriteStr;
use assert_fs::TempDir;

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

#[allow(dead_code)]
pub fn temp_dir() -> TempDir {
    TempDir::new().unwrap()
}

#[allow(dead_code)]
pub fn write<F: FileWriteStr>(file: F, data: &str) -> F {
    file.write_str(data).unwrap();
    file
}
