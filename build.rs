use std::path::Path;
use std::process::Command;

pub fn main() {
    if !Path::new(".git").exists() {
        return;
    }

    let output = Command::new("git")
        .arg("log")
        .arg("--max-count=1")
        .arg("--format=%h %cd")
        .arg("--abbrev=10")
        .arg("--date=short")
        .output()
        .unwrap();

    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut parts = stdout.split_whitespace();
    let mut next = || parts.next().unwrap();

    println!("cargo:rustc-env=BUILD_COMMIT_HASH={}", next());
    println!("cargo:rustc-env=BUILD_COMMIT_DATE={}", next());
}
