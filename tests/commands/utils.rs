use assert_cmd::assert::Assert;
use assert_cmd::crate_name;
use assert_cmd::prelude::*;
use assert_cmd::Command;
use rew::shell::Shell;
use std::env;
use std::process;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::Duration;

#[macro_export]
macro_rules! command_test {
    ($name:literal, { $($ident:ident : [ $($params:tt)* ]),+, }) => {
        mod tests {
            $($crate::command_test_case!($ident, $name, $($params)*);)*
        }
    };
}

#[macro_export]
macro_rules! command_test_case {
    ($ident:ident, $name:literal, cmd $($arg:literal)* assert $stdin:expr => $stdout:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::with_timeout(|| {
                $crate::utils::assert_command($name, &[$($arg,)*], $stdin)
                .success()
                .stdout($stdout)
                .stderr("");
            });
        }
    };
    ($ident:ident, $name:literal, cmd $($arg:literal)* assert $stdin:expr => err $stderr:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::with_timeout(|| {
                $crate::utils::assert_command($name, &[$($arg,)*], $stdin)
                    .failure()
                    .stderr($crate::utils::expand_err($name, $stderr));
            });
        }
    };
    ($ident:ident, $name:literal, sh $template:literal assert $stdin:expr => $stdout:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::with_timeout(|| {
              $crate::utils::assert_shell($template, $name, $stdin)
                    .success()
                    .stdout($stdout)
                    .stderr("");
            });
        }
    };
    ($ident:ident, $name:literal, sh $template:literal assert $stdin:expr => err $stderr:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::with_timeout(|| {
                $crate::utils::assert_shell($template, $name, $stdin)
                    .failure()
                    .stderr($crate::utils::expand_err($name, $stderr));
            });
        }
    };
}

pub fn assert_command(name: &str, args: &[&str], stdin: impl Into<Vec<u8>>) -> Assert {
    Command::cargo_bin(crate_name!())
        .unwrap()
        .arg(name)
        .args(args)
        .write_stdin(stdin)
        .assert()
}

pub fn assert_shell(template: &str, cmd: &str, stdin: impl Into<Vec<u8>>) -> Assert {
    let sh = env::var("SHELL").map(Shell::new).unwrap_or_default();
    let sh_kind = sh.kind();

    let bin = get_bin();
    let bin = sh_kind.normalize_path(&bin);

    let sh_cmd = template
        .replace("%bin%", &bin)
        .replace("%cmd%", &format!("{bin} {cmd}"));

    Command::from_std(sh.build_command(&sh_cmd))
        .write_stdin(stdin)
        .assert()
}

pub fn expand_err(cmd: &str, message: &str) -> String {
    format!("{} {cmd}: error: {message}", get_bin())
}

fn get_bin() -> String {
    process::Command::cargo_bin(crate_name!())
        .unwrap()
        .get_program()
        .to_string_lossy()
        .to_string()
}

// Inspired by `ntest::timeout`
// We cannot use `assert_cmd` timeouts because of https://github.com/assert-rs/assert_cmd/issues/167
// We could use `ntest::timeout` directly but it breaks test detection with VSCode and rust-analyzer
pub fn with_timeout(test: fn()) {
    let timeout = Duration::from_secs(5);
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        test();
        sender.send(()).expect("Could send to test receiver");
    });

    match receiver.recv_timeout(timeout) {
        Ok(()) => {}
        Err(RecvTimeoutError::Disconnected) => panic!(""), // Code inside the thread panicked
        Err(RecvTimeoutError::Timeout) => {
            panic!("Test run longer then allowed {} ms", timeout.as_millis())
        }
    }
}
