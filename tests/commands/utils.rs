use assert_cmd::assert::Assert;
use assert_cmd::crate_name;
use assert_cmd::prelude::*;
use assert_cmd::Command;
use rew::shell::Shell;
use std::env;
use std::process;
use std::time::Duration;

pub const TIMEOUT: Duration = Duration::from_secs(5);

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
        #[rstest::rstest]
        #[timeout($crate::utils::TIMEOUT)]
        fn $ident() {
            $crate::utils::assert_command($name, &[$($arg,)*], $stdin)
                .success()
                .stdout($stdout)
                .stderr("");
        }
    };
    ($ident:ident, $name:literal, cmd $($arg:literal)* assert $stdin:expr => err $stderr:expr) => {
        #[rstest::rstest]
        #[timeout($crate::utils::TIMEOUT)]
        fn $ident() {
            $crate::utils::assert_command($name, &[$($arg,)*], $stdin)
                .failure()
                .stderr($crate::utils::expand_err($name, $stderr));
        }
    };
    ($ident:ident, $name:literal, sh $template:literal assert $stdin:expr => $stdout:expr) => {
        #[rstest::rstest]
        #[timeout($crate::utils::TIMEOUT)]
        fn $ident() {
            $crate::utils::assert_shell($template, $name, $stdin)
                .success()
                .stdout($stdout)
                .stderr("");
        }
    };
    ($ident:ident, $name:literal, sh $template:literal assert $stdin:expr => err $stderr:expr) => {
        #[rstest::rstest]
        #[timeout($crate::utils::TIMEOUT)]
        fn $ident() {
            $crate::utils::assert_shell($template, $name, $stdin)
                .failure()
                .stderr($crate::utils::expand_err($name, $stderr));
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
