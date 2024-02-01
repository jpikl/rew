use assert_cmd::assert::Assert;
use assert_cmd::crate_name;
use assert_cmd::prelude::*;
use assert_cmd::Command;
use std::env;
use std::process;
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
    ($ident:ident, $name:literal, cmd $($arg:literal)* should $stdin:expr => $stdout:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::assert_command($name, &[$($arg,)*], $stdin)
                .success()
                .stdout($stdout)
                .stderr("");
        }
    };
    ($ident:ident, $name:literal, cmd $($arg:literal)* should $stdin:expr => err $stderr:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::assert_command($name, &[$($arg,)*], $stdin)
                .failure()
                .stderr($stderr);
        }
    };
    ($ident:ident, $name:literal, sh $template:literal should $stdin:expr => $stdout:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::assert_shell($template, $name, $stdin)
                .success()
                .stdout($stdout)
                .stderr("");
        }
    };
    ($ident:ident, $name:literal, sh $template:literal should $stdin:expr => err $stderr:expr) => {
        #[test]
        fn $ident() {
            $crate::utils::assert_shell($template, $name, $stdin)
                .failure()
                .stderr($stderr);
        }
    };
}

pub fn assert_command(name: &str, args: &[&str], stdin: impl Into<Vec<u8>>) -> Assert {
    Command::cargo_bin(crate_name!())
        .unwrap()
        .arg(name)
        .args(args)
        .write_stdin(stdin)
        .timeout(Duration::from_millis(500))
        .assert()
}

pub fn assert_shell(template: &str, cmd: &str, stdin: impl Into<Vec<u8>>) -> Assert {
    let bin = process::Command::cargo_bin(crate_name!()).unwrap();
    let bin_path = bin.get_program().to_string_lossy();

    let sh = env::var_os("SHELL").unwrap_or("sh".into());
    let sh_cmd = template.replace("%cmd%", &format!("{bin_path} {cmd}"));

    Command::new(sh)
        .arg("-c")
        .arg(sh_cmd)
        .write_stdin(stdin)
        .timeout(Duration::from_millis(500))
        .assert()
}
